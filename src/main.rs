use std::vec;

use macroquad::prelude::*;

const VIRTUAL_WIDTH: f32 = 650.0;
const VIRTUAL_HEIGHT: f32 = 450.0;
const ZOOM_FACTOR: f32 = 0.001;

/// How many pixels per world unit at the "reference" scale.
/// Tweak this to make your world larger or smaller.
const PIXELS_PER_UNIT: f32 = 32.0;

fn world_camera() -> Camera2D {
    let sw = screen_width();
    let sh = screen_height();

    // Half-extents in world units
    let half_w = sw / (2.0 * PIXELS_PER_UNIT);
    let half_h = sh / (2.0 * PIXELS_PER_UNIT);

    Camera2D {
        // Centre of the camera in world space
        target: vec2(0.0, 0.0),
        // zoom maps world units → NDC (-1..1).
        // macroquad's Camera2D zoom is (2/viewport_width, 2/viewport_height)
        // in world units, so:
        zoom: vec2(1.0 / half_w, -1.0 / half_h), // negative Y flips to match screen-Y
        ..Default::default()
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    // let render_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    // render_target.texture.set_filter(FilterMode::Linear);

    // let mut render_target_cam =
    //     Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    // render_target_cam.render_target = Some(render_target.clone());

    let mut pawns: Vec<Pawn> = vec![];
    let mut camera = world_camera();
    // let mut camera = Camera2D {
    //     // zoom: vec2(1., 1.),
    //     // zoom: vec2(1.0 / screen_width(), 1.0 / screen_height()),
    //     zoom: vec2(ZOOM_FACTOR, ZOOM_FACTOR * screen_width() / screen_height()),
    //     // zoom: vec2(0.01, 0.01),
    //     // offset: vec2(0.0, 100.0),
    //     // target: Vec2::ZERO,
    //     target: vec2(screen_width() / 2.0, screen_height() / 2.0),
    //     ..Default::default()
    // };
    let texture = load_texture("assets/infantry.png").await.unwrap();

    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Escape) {
            break;
        }
        set_camera(&camera);

        clear_background(LIGHTGRAY);

        draw_texture(&texture, 0.0, 0.0, WHITE);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 0.1, YELLOW);

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

        draw_fps();

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
    if is_mouse_button_down(MouseButton::Middle) {
        let mouse_delta = mouse_delta_position();
        camera.target += mouse_delta * 1000.0;
        println!("{:?}", camera)
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
