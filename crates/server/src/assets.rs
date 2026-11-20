//! The images a game uses: map images and units' top-face images, stored by content hash.
//!
//! An asset's id is the SHA-256 of its bytes, which does three jobs at once. The same image
//! added twice is stored once; an id can never come to mean a different image, so clients may
//! cache `/assets/{id}` forever; and a client that hashes a file itself knows the id before
//! the upload finishes, so it can send the command that uses the image straight after the
//! bytes without waiting for a reply.
//!
//! Only PNG and JPEG are accepted, read from each image's own header rather than from a file
//! name or a client's claim. The store never decodes pixels: whatever it holds is handed to
//! the browser to decode, so it only has to be sure the bytes really are an image of a format
//! every target browser can display.

use std::{collections::HashMap, fmt};

use bytes::Bytes;
use sha2::{Digest, Sha256};

/// The largest single image the server accepts.
pub const MAX_ASSET_BYTES: usize = 64 * 1024 * 1024;

/// Length of an asset id: SHA-256 as lowercase hex.
const ID_LENGTH: usize = 64;

/// An image id: the lowercase hex SHA-256 of the image's bytes.
pub type AssetId = String;

/// An image format the store accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageKind {
    Png,
    Jpeg,
}

impl ImageKind {
    pub fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }

    /// Extension for this image's entry in a save file.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

/// One stored image. Cloning shares the bytes, so serving a 16 MB map copies nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub kind: ImageKind,
    pub width: u32,
    pub height: u32,
    pub bytes: Bytes,
}

/// Why the store would not take an image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetError {
    TooLarge { bytes: usize },
    /// Not a format the store accepts.
    Unsupported,
    /// The right magic bytes, but the header that follows makes no sense.
    Malformed,
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { bytes } => write!(
                f,
                "image is {bytes} bytes, over the {MAX_ASSET_BYTES} byte limit"
            ),
            Self::Unsupported => write!(f, "not a PNG or JPEG image"),
            Self::Malformed => write!(f, "image header is damaged"),
        }
    }
}

impl std::error::Error for AssetError {}

/// Every image one game holds, keyed by content hash. Append-only within a game: ids in the
/// board, in a recorded turn and in an undo step must all keep resolving.
#[derive(Debug, Default)]
pub struct AssetStore {
    by_id: HashMap<AssetId, Asset>,
}

impl AssetStore {
    /// Store an image and return its id. An image already held is not stored twice.
    pub fn insert(&mut self, bytes: Bytes) -> Result<AssetId, AssetError> {
        if bytes.len() > MAX_ASSET_BYTES {
            return Err(AssetError::TooLarge { bytes: bytes.len() });
        }
        let (kind, width, height) = probe(&bytes)?;
        let id = asset_id(&bytes);
        self.by_id.entry(id.clone()).or_insert(Asset {
            kind,
            width,
            height,
            bytes,
        });
        Ok(id)
    }

    pub fn get(&self, id: &str) -> Option<&Asset> {
        self.by_id.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// The id of these bytes: their SHA-256 as lowercase hex. Clients compute the same id with
/// `crypto.subtle.digest`, so this must stay plain SHA-256 of the file as uploaded.
pub fn asset_id(bytes: &[u8]) -> AssetId {
    let digest = Sha256::digest(bytes);
    let mut id = String::with_capacity(ID_LENGTH);
    for byte in digest.iter() {
        // Hex digits, two per byte, without pulling in a hex crate.
        id.push(char::from_digit((byte >> 4) as u32, 16).expect("a nibble is a hex digit"));
        id.push(char::from_digit((byte & 0xf) as u32, 16).expect("a nibble is a hex digit"));
    }
    id
}

/// Whether this string could be an asset id. Checked before an id from outside is used to
/// look anything up, so that ids can later name files on disk without a traversal to worry
/// about.
pub fn is_asset_id(id: &str) -> bool {
    id.len() == ID_LENGTH
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// What an image's header says it is: format, width and height.
fn probe(bytes: &[u8]) -> Result<(ImageKind, u32, u32), AssetError> {
    if bytes.starts_with(&PNG_MAGIC) {
        return png_size(bytes);
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        return jpeg_size(bytes);
    }
    Err(AssetError::Unsupported)
}

const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// IHDR is always a PNG's first chunk, so the size sits at a fixed offset: the 8-byte
/// signature, the chunk's 4-byte length, its 4-byte type, then width and height big-endian.
fn png_size(bytes: &[u8]) -> Result<(ImageKind, u32, u32), AssetError> {
    if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
        return Err(AssetError::Malformed);
    }
    let read = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().expect("4 bytes"));
    let (width, height) = (read(16), read(20));
    if width == 0 || height == 0 {
        return Err(AssetError::Malformed);
    }
    Ok((ImageKind::Png, width, height))
}

/// A JPEG's size is in its start-of-frame segment, which can sit behind any number of other
/// segments, so the markers have to be walked. Each is `0xff`, a marker byte, then a
/// big-endian length that counts itself.
fn jpeg_size(bytes: &[u8]) -> Result<(ImageKind, u32, u32), AssetError> {
    let mut at = 2;
    while at + 3 < bytes.len() {
        if bytes[at] != 0xff {
            return Err(AssetError::Malformed);
        }
        let marker = bytes[at + 1];
        match marker {
            // Padding before a marker.
            0xff => {
                at += 1;
                continue;
            }
            // Markers that carry no payload.
            0x01 | 0xd0..=0xd9 => {
                at += 2;
                continue;
            }
            _ => {}
        }
        let length = u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]) as usize;
        if length < 2 {
            return Err(AssetError::Malformed);
        }
        // Start of frame, in any of its forms: baseline, progressive, lossless or
        // hierarchical. The other 0xc_ markers describe the coding tables instead.
        if matches!(marker, 0xc0..=0xcf if !matches!(marker, 0xc4 | 0xc8 | 0xcc)) {
            // Sample precision, then height and width.
            let frame = at + 4;
            if bytes.len() < frame + 5 {
                return Err(AssetError::Malformed);
            }
            let height = u16::from_be_bytes([bytes[frame + 1], bytes[frame + 2]]) as u32;
            let width = u16::from_be_bytes([bytes[frame + 3], bytes[frame + 4]]) as u32;
            if width == 0 || height == 0 {
                return Err(AssetError::Malformed);
            }
            return Ok((ImageKind::Jpeg, width, height));
        }
        at += 2 + length;
    }
    Err(AssetError::Malformed)
}

/// A PNG header of the given size, with no image data: enough for the store, which never
/// decodes pixels.
#[cfg(test)]
pub(crate) fn test_png(width: u32, height: u32) -> Bytes {
    let mut bytes = Vec::from(PNG_MAGIC);
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    // Bit depth, colour type, compression, filter and interlace.
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    Bytes::from(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A JPEG header of the given size: the start of image, one other segment to walk past,
    /// then a baseline start of frame.
    fn test_jpeg(width: u16, height: u16) -> Bytes {
        let mut bytes = vec![0xff, 0xd8];
        // An application segment, whose length must be stepped over to find the frame.
        bytes.extend_from_slice(&[0xff, 0xe0, 0x00, 0x04, 0x00, 0x00]);
        bytes.extend_from_slice(&[0xff, 0xc0, 0x00, 0x11, 8]);
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&width.to_be_bytes());
        Bytes::from(bytes)
    }

    #[test]
    fn an_id_is_the_sha256_of_the_bytes() {
        // The published digests of these inputs, so a change of hash cannot pass unnoticed:
        // clients compute this id themselves.
        assert_eq!(
            asset_id(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            asset_id(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn ids_are_recognised() {
        assert!(is_asset_id(&asset_id(b"anything")));
        assert!(!is_asset_id(""));
        assert!(!is_asset_id(&"a".repeat(63)));
        assert!(
            !is_asset_id(&"A".repeat(64)),
            "uppercase is not the hex an id is written in"
        );
        assert!(!is_asset_id(&"../".repeat(21)));
    }

    #[test]
    fn a_png_header_gives_its_size() {
        let mut store = AssetStore::default();
        let id = store.insert(test_png(4096, 2867)).unwrap();
        let asset = store.get(&id).unwrap();
        assert_eq!(
            (asset.kind, asset.width, asset.height),
            (ImageKind::Png, 4096, 2867)
        );
    }

    #[test]
    fn a_jpeg_header_gives_its_size() {
        let mut store = AssetStore::default();
        let id = store.insert(test_jpeg(640, 480)).unwrap();
        let asset = store.get(&id).unwrap();
        assert_eq!(
            (asset.kind, asset.width, asset.height),
            (ImageKind::Jpeg, 640, 480)
        );
    }

    #[test]
    fn the_same_image_is_stored_once() {
        let mut store = AssetStore::default();
        let first = store.insert(test_png(8, 8)).unwrap();
        let again = store.insert(test_png(8, 8)).unwrap();
        assert_eq!(first, again);
        assert_eq!(store.len(), 1);

        // A different image is a different id.
        let other = store.insert(test_png(8, 9)).unwrap();
        assert_ne!(first, other);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn other_formats_are_refused() {
        let mut store = AssetStore::default();
        // A GIF, a TIFF and something that is not an image at all.
        for bytes in [&b"GIF89a....."[..], &b"II*\0...."[..], &b"<svg/>"[..]] {
            let error = store.insert(Bytes::copy_from_slice(bytes)).unwrap_err();
            assert_eq!(error, AssetError::Unsupported, "{bytes:?}");
        }
        assert!(store.is_empty());
    }

    #[test]
    fn damaged_headers_are_refused() {
        let mut store = AssetStore::default();
        // A PNG signature with nothing behind it, and one whose first chunk is not IHDR.
        let truncated = Bytes::from(Vec::from(PNG_MAGIC));
        assert_eq!(store.insert(truncated).unwrap_err(), AssetError::Malformed);

        let mut wrong_chunk = test_png(8, 8).to_vec();
        wrong_chunk[12..16].copy_from_slice(b"iCCP");
        let wrong_chunk = Bytes::from(wrong_chunk);
        assert_eq!(store.insert(wrong_chunk).unwrap_err(), AssetError::Malformed);

        // A zero dimension would make a map plane with no extent.
        assert_eq!(
            store.insert(test_png(0, 8)).unwrap_err(),
            AssetError::Malformed
        );

        // A JPEG that never reaches a start-of-frame marker.
        let headless = Bytes::from_static(&[0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x00, 0x00]);
        assert_eq!(store.insert(headless).unwrap_err(), AssetError::Malformed);
        assert!(store.is_empty());
    }

    #[test]
    fn oversized_images_are_refused() {
        let mut store = AssetStore::default();
        let huge = Bytes::from(vec![0u8; MAX_ASSET_BYTES + 1]);
        assert_eq!(
            store.insert(huge).unwrap_err(),
            AssetError::TooLarge {
                bytes: MAX_ASSET_BYTES + 1
            },
            "checked before the header, so a huge upload is dropped without being parsed"
        );
    }
}
