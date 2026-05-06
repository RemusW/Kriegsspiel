#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod camera;
mod sprite;

use crate::camera::Camera;
use crate::sprite::{Sprite, Pawn};
use std::vec;

use macroquad::prelude::*;

const PIXELS_PER_UNIT: f32 = 1.0;

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut pawns: Vec<Pawn> = vec![];
    // let mut camera = world_camera();
    let mut camera = Camera::new();

    let cavalry = load_texture("assets/infantry.png").await.unwrap();
    cavalry.set_filter(FilterMode::Linear);
    let mut cavalry = Sprite::new(cavalry);
    // cavalry.set_scale(5.0, 5.0);

    let farley = load_texture("assets/farley.png").await.unwrap();
    farley.set_filter(FilterMode::Linear);
    let mut farley = Sprite::new(farley);
    farley.set_size(0.1, -0.1);
    // farley.set_scale(3200.0, -5500.0);

    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Escape) {
            break;
        }
        set_camera(&camera.to_macroquad());

        clear_background(LIGHTGRAY);

        farley.draw_default();
        cavalry.draw_default();

        // draw world grid
        // draw_grid(camera.target, screen_width() / 2.0, screen_height() / 2.0);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 500.0, 300.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_line(-0.4, 0.4, -0.8, 0.9, 10.0, BLUE);

        for ele in pawns.iter() {
            ele.draw();
        }
        spawn_pawn(&mut pawns, &camera, &cavalry);

        camera.update();

        set_default_camera();

        // Debug information
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

        next_frame().await
    }
}

fn pick_pawn(pawns: &Vec<Pawn>, camera: &Camera) {
    let mouse_pos = mouse_position();
    let mouse_pos = camera.screen_to_world(mouse_pos.into());
    for pawn in pawns.iter() {
        pawn.contains_point(mouse_pos);
    }
}

fn spawn_pawn(pawns: &mut Vec<Pawn>, camera: &Camera, sprite: &Sprite) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let position = camera.screen_to_world(mouse_position().into());
        let mut pawn = Pawn::new(position, sprite.clone());
        pawn.set_scale(0.1, 0.1);
        pawns.push(pawn);
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


