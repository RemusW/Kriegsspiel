use std::vec;

use macroquad::{prelude::*, text};

/// How many pixels per world unit at the "reference" scale.
/// Tweak this to make your world larger or smaller.
const PIXELS_PER_UNIT: f32 = 10.0;

fn world_camera() -> Camera2D {
    let sw = screen_width();
    let sh = screen_height();

    // Half-extents in world units
    let half_w = sw / (2.0 * PIXELS_PER_UNIT);
    let half_h = sh / (2.0 * PIXELS_PER_UNIT);

    Camera2D {
        target: vec2(0.0, 0.0),
        zoom: vec2(1.0 / half_w, -1.0 / half_h),
        ..Default::default()
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut pawns: Vec<Pawn> = vec![];
    let mut camera = world_camera();
    let texture = load_texture("assets/infantry.png").await.unwrap();
    texture.set_filter(FilterMode::Linear);
    let mut cavalry = Sprite::new(texture);
    cavalry.set_scale(5.0, 5.0);

    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Escape) {
            break;
        }
        set_camera(&camera);

        clear_background(LIGHTGRAY);

        // draw_texture(&texture, 0.0, 0.0, WHITE);
        // draw_texture_ex(
        //     &texture,
        //     0.0,
        //     0.0,
        //     WHITE,
        //     DrawTextureParams {
        //         // dest_size: Some(vec2(texture.width() / PIXELS_PER_UNIT, texture.height() / PIXELS_PER_UNIT)), // resize to this world size
        //         dest_size: Some(vec2(
        //             texture.width() / texture.height() * 5.0,
        //             texture.height() / texture.width() * 5.0,
        //         )),
        //         source: None, // None = full texture, or Some(Rect{...}) for a sprite sheet
        //         rotation: 0.0, // radians
        //         flip_x: false,
        //         flip_y: false,
        //         pivot: None, // rotation pivot, defaults to center
        //     },
        // );
        cavalry.draw();

        // draw world grid
        draw_grid(
            &camera,
            camera.target,
            screen_width() / 2.0,
            screen_height() / 2.0,
        );

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 500.0, 300.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_line(-0.4, 0.4, -0.8, 0.9, 10.0, BLUE);

        for ele in pawns.iter() {
            ele.draw();
        }
        spawn_pawn(&mut pawns);

        update_camera(&mut camera);

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

        next_frame().await
    }
}

struct Pawn {
    transform: Affine2,
}

impl Pawn {
    fn new(position: Vec2) -> Self {
        Pawn {
            transform: Affine2::from_translation(position),
        }
    }

    fn draw(&self) {
        let translation = self.transform.translation;
        draw_rectangle(translation.x, translation.y, 10.0, 10.0, RED);
        // println!("{:?}", translation);
    }
}

fn spawn_pawn(pawns: &mut Vec<Pawn>) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let position = Vec2::from(mouse_position());
        let pawn = Pawn::new(position);
        pawns.push(pawn);
    }
}

fn update_camera(camera: &mut Camera2D) {
    let sw = screen_width();
    let sh = screen_height();
    let half_w = sw / (2.0 * PIXELS_PER_UNIT);
    let half_h = sh / (2.0 * PIXELS_PER_UNIT);
    // camera.zoom = vec2(1.0 / half_w, -1.0 / half_h);

    if is_mouse_button_down(MouseButton::Middle) {
        let mouse_delta = mouse_delta_position();
        camera.target.x += mouse_delta.x * 10.0;
        camera.target.y -= mouse_delta.y * 10.0;
        // println!("{:?}", camera)
    }

    // let (_x, y) = mouse_wheel();
    // if y != 0.0 {
    //     // Normalize mouse wheel values is browser (chromium: 53, firefox: 3)
    //     #[cfg(target_arch = "wasm32")]
    //     let y = if y < 0.0 {
    //         -1.0
    //     } else if y > 0.0 {
    //         1.0
    //     } else {
    //         0.0
    //     };
    //     if is_key_down(KeyCode::LeftControl) {
    //         camera.zoom += 1.1f32.powf(y);
    //     }
    // }
    match mouse_wheel() {
        (_x, y) if y != 0.0 => {
            // Normalize mouse wheel values is browser (chromium: 53, firefox: 3)
            #[cfg(target_arch = "wasm32")]
            let y = if y < 0.0 {
                -1.0
            } else if y > 0.0 {
                1.0
            } else {
                0.0
            };
            if is_key_down(KeyCode::LeftControl) {
                camera.zoom *= 1.1f32.powf(y);
            }
            // camera.zoom += y;
        }
        _ => (),
    }
}

fn draw_grid(camera: &Camera2D, camera_target: Vec2, half_w: f32, half_h: f32) {
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

#[derive(Debug, Clone)]
struct Sprite {
    texture: Texture2D,
    transform: Transform,
}

impl Sprite {
    fn new(texture: Texture2D) -> Self {
        Self {
            transform: Transform::default(),
            texture: texture,
        }
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.transform.pos.x = x;
        self.transform.pos.y = y;
    }

    fn set_scale(&mut self, x: f32, y: f32) {
        self.transform.scale.x = x;
        self.transform.scale.y = y;
    }

    fn draw(&self) {
        let aspect = self.texture.width() / self.texture.height();
        draw_texture_ex(
            &self.texture,
            self.transform.pos.x,
            self.transform.pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    self.transform.scale.x * aspect,
                    self.transform.scale.y / aspect,
                )),
                source: None,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}

#[derive(Debug, Clone)]
struct Transform {
    pos: Vec2,
    rotation: f32,
    scale: Vec2,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}
