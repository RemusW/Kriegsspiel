//! Reading and writing save files: one self-contained file holding the document and every
//! image it names.
//!
//! A save is a zip, the same approach as `.docx`, `.ora` and `.usdz`, laid out as:
//!
//! ```text
//! mimetype                  the media type, stored, first: identifies the file by its bytes
//! manifest.json             the whole document, plus an entry per image in the archive
//! assets/<asset id>.png     one entry per image, stored
//! ```
//!
//! Images go in uncompressed: PNG and JPEG are compressed already, so deflating them costs
//! time for nothing. Only `manifest.json` is deflated, and it is the only part that is not
//! an image: at a few hundred kilobytes against a map image of many megabytes, the document
//! is a rounding error in the file, which is why it can stay readable JSON.
//!
//! This module knows only [`Doc`] and [`AssetStore`]: bytes in, bytes out, nothing of the
//! server around it. That keeps its tests plain and means it could move to a crate of its own
//! unchanged if a tool outside the server ever needs to read a save.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::{Cursor, Read, Write},
};

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    assets::{AssetError, AssetId, AssetStore, MAX_ASSET_BYTES, is_asset_id},
    model::{Board, Doc, Turn},
};

/// The media type of a save file: written into the archive, sent as the `Content-Type` of a
/// download, and offered in a file dialog. The `+zip` suffix tells any tool that does not
/// know the format that it may still open it as a zip.
pub const SAVE_MIME: &str = "application/vnd.kriegsspiel+zip";

/// The format this build writes. [`unpack`] refuses a file from a newer build rather than
/// loading it with the parts it does not understand silently dropped.
pub const FORMAT: u32 = 1;

/// The largest save file the server accepts, which has to hold several images at
/// [`MAX_ASSET_BYTES`] each.
pub const MAX_SAVE_BYTES: usize = 256 * 1024 * 1024;

/// The largest `manifest.json` a save may hold once inflated. Far above any real document, which
/// is a few kilobytes per hundred units; it exists so a small, highly compressed entry cannot
/// expand into gigabytes of memory.
pub const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;

const MANIFEST_PATH: &str = "manifest.json";
const MIMETYPE_PATH: &str = "mimetype";
const ASSETS_DIR: &str = "assets/";

/// What `manifest.json` holds. Everything in a save that is not an image is here, so the
/// things still to come (per-unit properties, script source, team messages) are added to this
/// one struct and to nothing else.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: u32,
    pub board: Board,
    #[serde(default)]
    pub turns: Vec<Turn>,
    /// Asset id to what the archive holds under it. Written so that a reader can list the
    /// images without decoding them; the bytes remain the truth, and [`unpack`] reads each
    /// image's own header rather than trusting these.
    #[serde(default)]
    pub assets: BTreeMap<AssetId, AssetInfo>,
}

/// One image, as the manifest describes it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub mime: String,
    pub width: u32,
    pub height: u32,
    /// Where the image sits inside the archive.
    pub path: String,
}

/// Why a save could not be written or read.
#[derive(Debug)]
pub enum SaveError {
    /// Not a zip, or an entry in it could not be read.
    Archive(zip::result::ZipError),
    Io(std::io::Error),
    /// There is no `manifest.json`, so this is some other zip.
    NoManifest,
    /// `manifest.json` is not the JSON this build expects.
    Manifest(serde_json::Error),
    /// `manifest.json` inflates to more than [`MAX_MANIFEST_BYTES`].
    ManifestTooLarge,
    /// Written by a newer build, which may have put things in it that would be lost.
    Future { format: u32 },
    /// An entry the manifest names is missing, misnamed, or does not hash to the id it is
    /// filed under.
    BadAsset { id: AssetId },
    /// An image in the archive is not one this build can serve.
    Image { id: AssetId, error: AssetError },
    /// The board or a recorded turn names an image the file does not hold.
    MissingAsset { id: AssetId },
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Archive(error) => write!(f, "not a readable save file: {error}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::NoManifest => write!(f, "no {MANIFEST_PATH}: this is not a save file"),
            Self::Manifest(error) => write!(f, "{MANIFEST_PATH} could not be read: {error}"),
            Self::ManifestTooLarge => {
                write!(f, "{MANIFEST_PATH} is over the {MAX_MANIFEST_BYTES} byte limit")
            }
            Self::Future { format } => write!(
                f,
                "saved by a newer version (format {format}, this build reads {FORMAT})"
            ),
            Self::BadAsset { id } => write!(f, "image {id} is missing from the file or damaged"),
            Self::Image { id, error } => write!(f, "image {id}: {error}"),
            Self::MissingAsset { id } => write!(f, "the game uses image {id}, which the file does not hold"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<zip::result::ZipError> for SaveError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::Archive(error)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(error: serde_json::Error) -> Self {
        Self::Manifest(error)
    }
}

/// Write a save: the live board, the recorded turns, and every image either of them names.
/// Images nothing refers to any more are left out, so a save does not carry the maps an
/// umpire tried and replaced. Undo history is not saved, so it is not asked for.
pub fn pack(board: &Board, turns: &[Turn], assets: &AssetStore) -> Result<Vec<u8>, SaveError> {
    // Gather the images first, so the manifest can be written before them and an umpire who
    // unzips a save finds the document at the top.
    let mut manifest = Manifest {
        format: FORMAT,
        board: board.clone(),
        turns: turns.to_vec(),
        assets: BTreeMap::new(),
    };
    let mut entries: Vec<(String, Bytes)> = Vec::new();
    for id in referenced(board, turns) {
        // The store is append-only and every command that names an image is checked against
        // it, so a missing one is a bug. Fail loudly rather than write a file that cannot be
        // loaded back.
        let asset = assets
            .get(&id)
            .ok_or_else(|| SaveError::MissingAsset { id: id.clone() })?;
        let path = format!("{ASSETS_DIR}{id}.{}", asset.kind.extension());
        manifest.assets.insert(
            id,
            AssetInfo {
                mime: asset.kind.mime().to_owned(),
                width: asset.width,
                height: asset.height,
                path: path.clone(),
            },
        );
        entries.push((path, asset.bytes.clone()));
    }

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

    // First, uncompressed and with no extra field, so that these bytes land at a fixed
    // offset and the format can be recognised without unzipping anything.
    zip.start_file(MIMETYPE_PATH, stored())?;
    zip.write_all(SAVE_MIME.as_bytes())?;

    zip.start_file(MANIFEST_PATH, deflated())?;
    zip.write_all(&serde_json::to_vec_pretty(&manifest)?)?;

    for (path, bytes) in entries {
        zip.start_file(path, stored())?;
        zip.write_all(&bytes)?;
    }

    Ok(zip.finish()?.into_inner())
}

/// Read a save into a document and the images it uses. The document comes back with no undo
/// history, because a fresh [`Doc`] has none: loading a file is not an edit anyone can take
/// back, and the steps in an old history would refer to a board that is gone.
pub fn unpack(bytes: &[u8]) -> Result<(Doc, AssetStore), SaveError> {
    let mut zip = ZipArchive::new(Cursor::new(bytes))?;

    let manifest: Manifest = {
        let mut entry = match zip.by_name(MANIFEST_PATH) {
            Ok(entry) => entry,
            Err(zip::result::ZipError::FileNotFound) => return Err(SaveError::NoManifest),
            Err(error) => return Err(error.into()),
        };
        // Capped like the images below, so a deflated entry cannot expand without limit.
        let mut text = Vec::new();
        entry
            .by_ref()
            .take(MAX_MANIFEST_BYTES as u64 + 1)
            .read_to_end(&mut text)?;
        if text.len() > MAX_MANIFEST_BYTES {
            return Err(SaveError::ManifestTooLarge);
        }
        serde_json::from_slice(&text)?
    };
    if manifest.format > FORMAT {
        return Err(SaveError::Future {
            format: manifest.format,
        });
    }

    let mut assets = AssetStore::default();
    for (id, info) in &manifest.assets {
        // An id from a file is used to look an entry up and could later name a file on disk.
        if !is_asset_id(id) || !info.path.starts_with(ASSETS_DIR) {
            return Err(SaveError::BadAsset { id: id.clone() });
        }
        let mut entry = zip
            .by_name(&info.path)
            .map_err(|_| SaveError::BadAsset { id: id.clone() })?;
        let mut data = Vec::new();
        // Capped one byte past the limit, so a deflated entry cannot expand without limit and
        // an oversized one still reaches the store's own size check below.
        entry
            .by_ref()
            .take(MAX_ASSET_BYTES as u64 + 1)
            .read_to_end(&mut data)?;

        // The bytes decide what the image is and the id is their hash, so a manifest that
        // disagrees with its own payload cannot pass a different image off as this one.
        let stored = assets
            .insert(Bytes::from(data))
            .map_err(|error| SaveError::Image {
                id: id.clone(),
                error,
            })?;
        if &stored != id {
            return Err(SaveError::BadAsset { id: id.clone() });
        }
    }

    let mut doc = Doc::default();
    doc.board = manifest.board;
    doc.turns = manifest.turns;

    for id in referenced(&doc.board, &doc.turns) {
        if !assets.contains(&id) {
            return Err(SaveError::MissingAsset { id });
        }
    }
    Ok((doc, assets))
}

/// Every image id the document names: the live board's and every recorded turn's. A turn may
/// still use a map image the live board has moved on from, so a save has to keep both.
fn referenced(board: &Board, turns: &[Turn]) -> BTreeSet<AssetId> {
    let boards = std::iter::once(board).chain(turns.iter().map(|turn| &turn.board));
    boards
        .flat_map(Board::images)
        .map(ToOwned::to_owned)
        .collect()
}

/// For images, which are compressed already.
fn stored() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
}

/// For the manifest, the one entry worth compressing.
fn deflated() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assets::test_png,
        model::{Side, UnitKind},
    };

    /// A game with two units, a map image, and one recorded turn.
    fn saved_game() -> (Doc, AssetStore) {
        let mut assets = AssetStore::default();
        let map = assets.insert(test_png(4096, 2867)).unwrap();
        let badge = assets.insert(test_png(64, 64)).unwrap();

        let mut doc = Doc::default();
        doc.board.map_image = Some(map);
        let id = doc.board.spawn(UnitKind::Infantry, Side::Blue, [1.0, 2.0]);
        doc.board.spawn(UnitKind::Cavalry, Side::Red, [3.0, 4.0]);
        doc.board.unit_mut(id).unwrap().pawn.image = Some(badge);
        doc.record_turn(1_700_000_000_000);
        (doc, assets)
    }

    #[test]
    fn a_save_round_trips() {
        let (doc, assets) = saved_game();
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();
        let (back, back_assets) = unpack(&file).unwrap();

        assert_eq!(back.board, doc.board);
        assert_eq!(back.turns, doc.turns);
        assert_eq!(back_assets.len(), assets.len());
        // The images come back byte for byte, under the same ids.
        for id in referenced(&doc.board, &doc.turns) {
            assert_eq!(
                back_assets.get(&id).map(|a| &a.bytes),
                assets.get(&id).map(|a| &a.bytes),
                "{id}"
            );
        }
    }

    #[test]
    fn a_save_is_a_zip_that_names_itself() {
        let (doc, assets) = saved_game();
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();

        assert_eq!(&file[..4], b"PK\x03\x04", "a zip, openable by any tool");
        // The media type sits at a fixed offset: the 30-byte local header, then the entry
        // name. This is what lets a dropped file be recognised without unzipping it.
        assert_eq!(&file[30..38], MIMETYPE_PATH.as_bytes());
        assert_eq!(&file[38..38 + SAVE_MIME.len()], SAVE_MIME.as_bytes());
    }

    #[test]
    fn the_manifest_is_readable_json() {
        let (doc, assets) = saved_game();
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();

        let mut zip = ZipArchive::new(Cursor::new(&file)).unwrap();
        let mut text = String::new();
        zip.by_name(MANIFEST_PATH)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(json["format"], FORMAT);
        assert_eq!(json["board"]["pawns"][0]["kind"], "Infantry");
        assert_eq!(json["turns"][0]["recordedAt"], 1_700_000_000_000u64);
        let map = doc.board.map_image.as_ref().unwrap();
        assert_eq!(json["assets"][map]["mime"], "image/png");
        assert_eq!(json["assets"][map]["width"], 4096);
        assert_eq!(json["assets"][map]["path"], format!("assets/{map}.png"));
    }

    #[test]
    fn images_are_stored_uncompressed_and_only_once() {
        let mut assets = AssetStore::default();
        let shared = assets.insert(test_png(64, 64)).unwrap();
        let mut doc = Doc::default();
        // Twenty units sharing one image: content addressing means one entry.
        for _ in 0..20 {
            let id = doc.board.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
            doc.board.unit_mut(id).unwrap().pawn.image = Some(shared.clone());
        }

        let file = pack(&doc.board, &doc.turns, &assets).unwrap();
        let mut zip = ZipArchive::new(Cursor::new(&file)).unwrap();
        let paths = entry_paths(&mut zip);
        assert_eq!(
            paths.iter().filter(|p| p.starts_with(ASSETS_DIR)).count(),
            1
        );

        let entry = zip.by_name(&format!("{ASSETS_DIR}{shared}.png")).unwrap();
        assert_eq!(entry.compression(), CompressionMethod::Stored);
    }

    #[test]
    fn a_save_leaves_out_images_nothing_uses() {
        let (mut doc, assets) = saved_game();
        let replaced = doc.board.map_image.take();
        assert!(replaced.is_some());
        // The recorded turn still holds the map, so it must still be saved.
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();
        let (_, back) = unpack(&file).unwrap();
        assert_eq!(back.len(), 2);

        // With no turn to hold it, the map image goes.
        doc.turns.clear();
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();
        let (_, back) = unpack(&file).unwrap();
        assert_eq!(back.len(), 1, "only the unit's image is still in use");
    }

    #[test]
    fn loading_a_save_starts_with_no_history() {
        let (mut doc, assets) = saved_game();
        doc.edit(7, |board| {
            board.spawn(UnitKind::Artillery, Side::Red, [9.0, 9.0]);
        });
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();

        let (mut back, _) = unpack(&file).unwrap();
        assert_eq!(back.board.pawns.len(), 3, "the edit itself is saved");
        assert_eq!(
            back.undo(7),
            crate::model::Step::Empty,
            "but it cannot be taken back in the loaded game"
        );
    }

    #[test]
    fn something_that_is_not_a_save_is_refused() {
        let not_a_zip = unpack(b"this is not a zip file at all").unwrap_err();
        assert!(matches!(not_a_zip, SaveError::Archive(_)), "{not_a_zip:?}");

        // A zip with no manifest in it: some other archive the umpire picked by mistake.
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file("notes.txt", stored()).unwrap();
        zip.write_all(b"hello").unwrap();
        let other_zip = zip.finish().unwrap().into_inner();
        assert!(matches!(
            unpack(&other_zip).unwrap_err(),
            SaveError::NoManifest
        ));
    }

    #[test]
    fn a_newer_format_is_refused_rather_than_half_read() {
        let (doc, assets) = saved_game();
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();
        let mut repacked = repack_manifest(&file, |json| {
            json["format"] = serde_json::json!(FORMAT + 1);
        });

        match unpack(&repacked).unwrap_err() {
            SaveError::Future { format } => assert_eq!(format, FORMAT + 1),
            other => panic!("{other:?}"),
        }

        // An older format is still read: a save must not stop loading when the format moves on.
        repacked = repack_manifest(&file, |json| {
            json["format"] = serde_json::json!(FORMAT);
        });
        assert!(unpack(&repacked).is_ok());
    }

    #[test]
    fn a_manifest_that_does_not_match_its_images_is_refused() {
        let (doc, assets) = saved_game();
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();
        let map = doc.board.map_image.clone().unwrap();

        // An entry filed under an id that is not the hash of its bytes.
        let swapped = repack_manifest(&file, |json| {
            let mut entry = json["assets"][&map].clone();
            entry["path"] = serde_json::json!(format!("{ASSETS_DIR}{map}.png"));
            json["assets"] = serde_json::json!({ "0".repeat(64): entry });
            json["board"]["mapImage"] = serde_json::json!("0".repeat(64));
        });
        assert!(matches!(
            unpack(&swapped).unwrap_err(),
            SaveError::BadAsset { .. }
        ));

        // A board that names an image the file does not hold.
        let dangling = repack_manifest(&file, |json| {
            json["board"]["mapImage"] = serde_json::json!("1".repeat(64));
        });
        match unpack(&dangling).unwrap_err() {
            SaveError::MissingAsset { id } => assert_eq!(id, "1".repeat(64)),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_manifest_over_the_limit_is_refused_before_it_is_parsed() {
        let (doc, assets) = saved_game();
        let file = pack(&doc.board, &doc.turns, &assets).unwrap();
        // Still valid JSON, and an unknown field would otherwise be ignored: only the size
        // can be what refuses it.
        let bloated = repack_manifest(&file, |json| {
            json["padding"] = serde_json::json!(" ".repeat(MAX_MANIFEST_BYTES));
        });
        assert!(matches!(
            unpack(&bloated).unwrap_err(),
            SaveError::ManifestTooLarge
        ));
    }

    /// Every entry in an archive, in the order it was written.
    fn entry_paths(zip: &mut ZipArchive<Cursor<impl AsRef<[u8]>>>) -> Vec<String> {
        zip.file_names()
            .map(|name| name.expect("a written archive lists its entries").into_owned())
            .collect()
    }

    /// Rewrite a save's manifest, keeping every other entry, to build the damaged files a
    /// save has to cope with.
    fn repack_manifest(file: &[u8], change: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
        let mut source = ZipArchive::new(Cursor::new(file)).unwrap();
        let mut json: serde_json::Value = {
            let mut text = String::new();
            source
                .by_name(MANIFEST_PATH)
                .unwrap()
                .read_to_string(&mut text)
                .unwrap();
            serde_json::from_str(&text).unwrap()
        };
        change(&mut json);

        let paths = entry_paths(&mut source);
        let mut out = ZipWriter::new(Cursor::new(Vec::new()));
        for path in paths {
            let mut data = Vec::new();
            source
                .by_name(&path)
                .unwrap()
                .read_to_end(&mut data)
                .unwrap();
            out.start_file(&path, stored()).unwrap();
            if path == MANIFEST_PATH {
                out.write_all(&serde_json::to_vec(&json).unwrap()).unwrap();
            } else {
                out.write_all(&data).unwrap();
            }
        }
        out.finish().unwrap().into_inner()
    }
}
