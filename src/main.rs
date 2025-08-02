use bevy::DefaultPlugins;
use bevy::prelude::*;

use crate::pawn::PawnPlugin;

mod pawn;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_systems(Startup, scene_setup)
        .add_plugins(PawnPlugin)
        // .add_systems(Update, (hello_world, (update_people, greet_people).chain()))
        .run();
}

fn scene_setup (mut commands: Commands) {
    commands.spawn(Camera2d::default());
}