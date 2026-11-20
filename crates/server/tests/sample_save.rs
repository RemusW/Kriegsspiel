//! The sample save in `client/assets` must keep loading.
//!
//! It was written from the format's description by a separate tool rather than by `pack`, so
//! this also checks that a save can be read back from something other than the writer that
//! made it, and that the public API is enough to read one.
//!
//! The sample is not in the repository (`client/assets/*.ksg` is ignored: saves are
//! generated, and one with a real map in it is tens of megabytes), so this passes by default
//! on a fresh clone and only has something to say where the file has been built.

use std::path::Path;

use kriegsspiel_server::save;

#[test]
fn the_sample_save_loads() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client/assets/sample.ksg");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("no sample save at {}, nothing to check", path.display());
        return;
    };

    let (doc, assets) = save::unpack(&bytes).expect("the sample save loads");
    assert_eq!(doc.board.pawns.len(), 4, "the demo board");
    assert_eq!(doc.turns.len(), 1, "and one recorded turn to step back to");

    let map = doc.board.map_image.as_deref().expect("it lays a map down");
    let asset = assets.get(map).expect("and carries that map with it");
    // Portrait, so opening the sample visibly reshapes the table from the blank 100 x 70.
    assert_eq!((asset.width, asset.height), (512, 768));
    assert_eq!(asset.kind.mime(), "image/png");
    // The map is synthetic and small on purpose: nothing here needs a real 16 MB survey map.
    assert!(bytes.len() < 64 * 1024, "{} bytes", bytes.len());

    // The recorded turn keeps the same map, which is why one copy of it is enough.
    assert_eq!(doc.turns[0].board.map_image.as_deref(), Some(map));
    assert_eq!(assets.len(), 1);
}
