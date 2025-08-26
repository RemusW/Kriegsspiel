// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
// mod demo;
mod actions;
#[cfg(feature = "dev")]
mod dev_tools;
mod interface;
mod menus;
mod screens;
mod theme;

use bevy::DefaultPlugins;
use bevy::input::mouse::MouseWheel;
use bevy::math::ops::powf;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::pawn::PawnPlugin;

mod pawn;

const CAMERA_MOVE_SPEED: f32 = 500.0;

fn main() -> AppExit {
    App::new()
        .add_systems(Startup, scene_setup)
        .add_plugins((
            DefaultPlugins,
            interface::plugin,
            MeshPickingPlugin,
            PawnPlugin,
            actions::plugin,
            EguiPlugin::default(),
            dev_tools::plugin,
        ))
        .add_systems(Update, move_camera)
        .add_systems(Update, zoom_camera)
        // .add_systems(Update, (hello_world, (update_people, greet_people).chain()))
        .run()
}

fn scene_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("farley.png")),
        RenderLayers::layer(1),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
    commands.spawn((Camera2d::default(), RenderLayers::from_layers(&[0, 1])));
    // asset_server.load("sprites/ball.png");
}

/// Update the camera position with keyboard inputs.
fn move_camera(
    mut camera: Single<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    let move_delta = direction.normalize_or_zero() * CAMERA_MOVE_SPEED * time.delta_secs();
    camera.translation += move_delta.extend(0.);
}

fn zoom_camera(
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
    mut scroll_evr: EventReader<MouseWheel>,
    time: Res<Time<Fixed>>,
) {
    let Ok((_, _, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let mut zoom_delta = 0.0;
    for ev in scroll_evr.read() {
        zoom_delta += ev.y; // ev.y is positive for scroll up
    }

    if zoom_delta.abs() > 0.0 {
        if let Projection::Orthographic(projection2d) = &mut *projection {
            if zoom_delta > 0.0 {
                projection2d.scale *= powf(4.0f32, time.delta_secs());
            } else {
                projection2d.scale *= powf(0.25f32, time.delta_secs());
            }
        }
    }
}
