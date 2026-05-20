#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod camera;
mod command;
mod sprite;
mod editor;

use crate::camera::Camera;
use crate::command::CommandManager;
use crate::editor::{EditorState, ToolMode, SelectionTool};
use crate::sprite::{Pawn, Sprite};
use std::cell::Cell;
use std::collections::HashMap;
use std::vec;

use egui_macroquad::egui;
use macroquad::prelude::*;
use uuid::Uuid;

const PIXELS_PER_UNIT: f32 = 1.0;


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
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut editor_state = EditorState::default();

    let mut command_manager = CommandManager::new();
    let mut world = World::new();

    let cavalry = load_texture("assets/infantry.png").await.unwrap();
    cavalry.set_filter(FilterMode::Linear);
    let cavalry = Sprite::new(cavalry);
    // cavalry.set_scale(5.0, 5.0);

    let farley = load_texture("assets/farley.png").await.unwrap();
    farley.set_filter(FilterMode::Linear);
    let mut farley = Sprite::new(farley);
    farley.set_size(1000.0, -1000.0);

    world.map = Some(farley);

    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Escape) {
            break;
        }
        set_camera(&world.camera.to_macroquad());

        clear_background(LIGHTGRAY);

        egui_macroquad::ui(|ctx| {
            egui::Window::new("Actions")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Mouse Mode");
                    ui.separator();
                    for mode in [ToolMode::Spawn, ToolMode::Selection] {
                        ui.selectable_value(&mut editor_state.tool_mode, mode, mode.label());
                    }
                });
        });

        if let Some(ref map) = world.map {
            map.draw_default();
        }

        // draw world grid
        // draw_grid(camera.target, screen_width() / 2.0, screen_height() / 2.0);

        draw_circle(0.0, 0.0, 50.0, WHITE);
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_line(-0.4, 0.4, -0.8, 0.9, 10.0, BLUE);


        for ele in world.pawn_manager.pawn_map.iter() {
            ele.1.draw();
        }

        editor_state.update(&mut world);

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
