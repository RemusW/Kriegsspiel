#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod camera;
mod command;
mod editor;
mod sprite;
mod asset;
mod menu;

use crate::asset::AssetStore;
use crate::camera::Camera;
use crate::command::CommandManager;
use crate::editor::{EditorState, SelectionTool, ToolMode};
use crate::menu::SceneManager;
use crate::sprite::{Pawn, Sprite};
use std::cell::Cell;
use std::collections::HashMap;
use std::{default, vec};

use egui_macroquad::egui;
use egui_macroquad::egui::Key::{D, O};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const PIXELS_PER_UNIT: f32 = 1.0;

#[macroquad::main("BasicShapes")]
async fn main() {
    // Singletons
    let mut scene_manager = SceneManager::new();
    let mut appctx = AppContext::default();

    loop {
        appctx.asset_store.process_pending().await;

        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Escape) {
            break;
        }
        set_camera(&appctx.world.camera.to_macroquad());

        scene_manager.run(&mut appctx);

        next_frame().await
    }
}

#[derive(Default)]
struct AppContext{
    command_manger: CommandManager,
    asset_store: AssetStore,
    world: World,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct World {
    pawn_manager: PawnManager,
    camera: Camera,
    map: Option<Sprite>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PawnManager {
    pawn_map: HashMap<Uuid, Pawn>,
}
impl PawnManager {
    pub fn new() -> Self {
        Self {
            pawn_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, pawn: Pawn) {
        self.pawn_map.insert(pawn.get_uid(), pawn);
    }

    pub fn get_pawns_from_uid(&self, uids: Vec<Uuid>) -> Vec<&Pawn> {
        let mut pawns: Vec<&Pawn> = Vec::new();
        for uid in uids.iter() {
            if let Some(p) = self.pawn_map.get(uid) {
                pawns.push(p);
            }
        }
        pawns
    }

    pub fn get_pawns_from_uid_mut(&mut self, uids: &[Uuid]) -> Vec<&mut Pawn> {
        self.pawn_map
            .iter_mut()
            .filter(|(id, _)| uids.contains(id))
            .map(|(_, pawn)| pawn)
            .collect()
    }
}

fn spawn_pawn(world: &mut World, sprite: &Sprite) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let position = world.camera.screen_to_world(mouse_position().into());
        let mut pawn = Pawn::new(world, position, sprite.clone());
        pawn.set_scale(0.1, 0.1);
        world.pawn_manager.add(pawn);
    }
}

fn draw_grid(camera_target: Vec2, half_w: f32, half_h: f32) {
    let spacing = 5.0;
    let line_color = Color::new(0.4, 0.4, 0.4, 1.0);

    let x_start = ((camera_target.x - half_w) / spacing).floor() as i32 - 1;
    let x_end = ((camera_target.x + half_w) / spacing).ceil() as i32 + 1;
    let y_start = ((camera_target.y - half_h) / spacing).floor() as i32 - 1;
    let y_end = ((camera_target.y + half_h) / spacing).ceil() as i32 + 1;

    for x in x_start..=x_end {
        let wx = x as f32 * spacing;
        draw_line(
            wx,
            y_start as f32 * spacing,
            wx,
            y_end as f32 * spacing,
            1.0 / PIXELS_PER_UNIT,
            line_color,
        );
    }
    for y in y_start..=y_end {
        let wy = y as f32 * spacing;
        draw_line(
            x_start as f32 * spacing,
            wy,
            x_end as f32 * spacing,
            wy,
            1.0 / PIXELS_PER_UNIT,
            line_color,
        );
    }
}
