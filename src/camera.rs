use macroquad::{prelude::*};

#[derive(Debug, Clone)]
pub struct Camera {
    pub target: Vec2,
    pub zoom_factor: f32,
    orthographic_size: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            target: Vec2::ZERO,
            zoom_factor: 100.0,
            orthographic_size: 1.0,
        }
    }

    pub fn to_macroquad(&self) -> Camera2D {
        let ppu = screen_height() / (self.orthographic_size * self.zoom_factor);
        let half_w = screen_width() / (2.0 * ppu);
        let half_h = screen_height() / (2.0 * ppu);
        Camera2D {
            target: self.target,
            zoom: vec2(1.0 / half_w, -1.0 / half_h),
            ..Default::default()
        }
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.to_macroquad().screen_to_world(screen_pos)
    }

    pub fn update(&mut self) {
        let pan_speed = 10.0;
        if is_mouse_button_down(MouseButton::Middle) {
            let mouse_delta = mouse_delta_position();
            self.target.x += mouse_delta.x * pan_speed;
            self.target.y -= mouse_delta.y * pan_speed;
        }
        if is_key_down(KeyCode::W) {
            self.target.y += pan_speed;
        }
        if is_key_down(KeyCode::S) {
            self.target.y -= pan_speed;
        }
        if is_key_down(KeyCode::A) {
            self.target.x -= pan_speed;
        }
        if is_key_down(KeyCode::D) {
            self.target.x += pan_speed;
        }

        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let zoom_speed = 1.3;
            if scroll > 0.0 {
                self.zoom_factor *= zoom_speed;
            } else {
                self.zoom_factor /= zoom_speed;
            }

            // Clamp zoom range
            self.zoom_factor = self.zoom_factor.clamp(0.1, 1000.0);
            println!("{:?}", self.zoom_factor);
        }
    }
}
