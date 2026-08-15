use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{AppContext, World, asset::AssetStore};

#[derive(Serialize)]
pub struct SaveFileRef<'a> {
    world: &'a World,
    manifest: &'a HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct SaveFileOwned {
    world: World,
    manifest: HashMap<String, String>,
}

pub fn create_save_file(world: &World, asset_store: &AssetStore) {
    let save_file = SaveFileRef {
        world: &world,
        manifest: &asset_store.get_manifest(),
    };
    match ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()) {
        Ok(data) => match std::fs::write("assets/world.ron", data) {
            Ok(()) => println!("Saved world to assets/world.ron"),
            Err(e) => eprintln!("Failed to write world: {e}"),
        },
        Err(e) => eprintln!("Failed to serialize world: {e}"),
    }
}

pub fn load_save_file(ctx: &mut AppContext, save_file: SaveFileOwned) {
    ctx.world = save_file.world;
    ctx.asset_store.load_manifest(save_file.manifest);
}
