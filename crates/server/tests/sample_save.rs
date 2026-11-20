//! The sample save in `client/assets` must keep loading.
//!
//! It was written from the format's description by a separate tool rather than by `pack`, so
//! this also checks that a save can be read back from something other than the writer that
//! made it, and that the public API is enough to read one.

use std::path::Path;

use kriegsspiel_server::save;

#[test]
fn the_sample_save_loads() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client/assets/sample.ksg");
    let bytes = std::fs::read(&path).expect("the sample save is in the repository");

    let (doc, assets) = save::unpack(&bytes).expect("the sample save loads");
    assert_eq!(doc.board.pawns.len(), 4, "the demo board");
    assert_eq!(doc.turns.len(), 1, "and one recorded turn to step back to");

    let map = doc.board.map_image.as_deref().expect("it lays a map down");
    let asset = assets.get(map).expect("and carries that map with it");
    assert_eq!((asset.width, asset.height), (3659, 5388));
    assert_eq!(asset.kind.mime(), "image/png");
    assert_eq!(asset.bytes.len(), 15_737_445);

    // The recorded turn keeps the same map, which is why one copy of it is enough.
    assert_eq!(doc.turns[0].board.map_image.as_deref(), Some(map));
    assert_eq!(assets.len(), 1);
}
