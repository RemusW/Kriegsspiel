#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod camera;
mod command;
mod editor;
mod sprite;
mod seralize;
mod asset;

use crate::asset::AssetStore;
use crate::camera::Camera;
use crate::command::CommandManager;
use crate::editor::{EditorState, SelectionTool, ToolMode};
use crate::sprite::{Pawn, Sprite};
use std::cell::Cell;
use std::collections::HashMap;
use std::vec;

use egui_macroquad::egui;
use egui_macroquad::egui::Key::{D, O};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const PIXELS_PER_UNIT: f32 = 1.0;

#[derive(Debug, Serialize, Deserialize)]
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

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut editor_state = EditorState::default();

    let mut command_manager = CommandManager::new();
    let mut world = World::new();
    let mut asset_store = AssetStore::default();

    let cavalry = asset_store.load_texture("cavalry", "assets/cavalry.png");
    let farley = asset_store.load_texture("farley", "assets/farley.png");
    asset_store.process_pending().await;

    let cavalry = Sprite::new(cavalry, &asset_store);
    // cavalry.set_scale(5.0, 5.0);

    let mut farley = Sprite::new(farley, &asset_store);
    farley.set_size(1000.0, -1000.0);

    world.map = Some(farley);
    

    loop {
        asset_store.process_pending().await;

        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Escape) {
            break;
        }
        set_camera(&world.camera.to_macroquad());

        clear_background(LIGHTGRAY);

        let mut save_world = false;
        egui_macroquad::ui(|ctx| {
            egui::Window::new("Actions")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Mouse Mode");
                    ui.separator();
                    for mode in [ToolMode::Spawn, ToolMode::Move] {
                        ui.selectable_value(&mut editor_state.tool_mode, mode, mode.label());
                    }
                    ui.separator();
                    if ui.button("Save World").clicked() {
                        save_world = true;
                    }
                });
        });

        if save_world {
            // JSON
            // match serde_json::to_string_pretty(&world) {
            //     Ok(data) => match std::fs::write("assets/world.json", data) {
            //         Ok(()) => println!("Saved world to assets/world.json"),
            //         Err(e) => eprintln!("Failed to write world: {e}"),
            //     },
            //     Err(e) => eprintln!("Failed to serialize world: {e}"),
            // }

            // RON — comment out the JSON block above and uncomment this to switch
            match ron::ser::to_string_pretty(&world, ron::ser::PrettyConfig::default()) {
                Ok(data) => match std::fs::write("assets/world.ron", data) {
                    Ok(()) => println!("Saved world to assets/world.ron"),
                    Err(e) => eprintln!("Failed to write world: {e}"),
                },
                Err(e) => eprintln!("Failed to serialize world: {e}"),
            }
        }

        if let Some(ref map) = world.map {
            map.draw_default();
        }

        // draw world grid
        // draw_grid(camera.target, screen_width() / 2.0, screen_height() / 2.0);

        editor_state.update(&mut world, &mut command_manager);
        if editor_state.tool_mode == ToolMode::Spawn {
            spawn_pawn(&mut world, &cavalry);
        }

        draw_circle(0.0, 0.0, 50.0, WHITE);
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_line(-0.4, 0.4, -0.8, 0.9, 10.0, BLUE);

        // Draw highlight box around pawn if selected
        let pawns = editor_state.selection.get_selected_pawns(&world);
        for pawn in pawns.iter() {
            pawn.draw_highlight();
        }

        for ele in world.pawn_manager.pawn_map.iter() {
            ele.1.draw();
        }

        world.camera.update();

        set_default_camera();

        draw_text(
            &format!(
                "Window: {}x{}  |  World units visible: {:.1} x {:.1}",
                screen_width() as i32,
                screen_height() as i32,
                screen_width() / PIXELS_PER_UNIT,
                screen_height() / PIXELS_PER_UNIT,
            ),
            10.0,
            20.0,
            20.0,
            WHITE,
        );
        draw_text(&format!("{:?}", mouse_position()), 10.0, 40.0, 20.0, WHITE);

        // Debug information
        egui_macroquad::draw();

        next_frame().await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct World {
    pawn_manager: PawnManager,
    camera: Camera,
    map: Option<Sprite>,
}

impl World {
    fn new() -> Self {
        Self {
            pawn_manager: PawnManager::new(),
            camera: Camera::new(),
            map: None,
        }
    }
}

fn pick_pawn(pawns: &Vec<Pawn>, camera: &Camera) {
    let mouse_pos = mouse_position();
    let mouse_pos = camera.screen_to_world(mouse_pos.into());
    for pawn in pawns.iter() {
        pawn.contains_point(mouse_pos);
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
