use std::{default, vec::Splice};

use macroquad::prelude::*;

use crate::PIXELS_PER_UNIT;

#[derive(Debug, Clone, Copy, Default)]
pub enum Pivot {
    #[default]
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Pivot {
    // Returns the offset to subtract from pos to get top-left draw position
    pub fn offset(&self, size: Vec2) -> Vec2 {
        match self {
            Pivot::Center => size * 0.5,
            Pivot::TopLeft => Vec2::ZERO,
            Pivot::TopRight => vec2(size.x, 0.0),
            Pivot::BottomLeft => vec2(0.0, size.y),
            Pivot::BottomRight => size,
        }
    }
}

pub enum SpriteImageMode {
    Center,
    Fill,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    texture: Texture2D,
    custom_size: Option<Vec2>,
    pivot: Pivot,
    // flip_x: bool,
    // flip_y: bool,
}

// impl Default for Sprite {
//     fn default() -> Self {
//         Self { texture: None, custom_size: (), pivot: (), flip_x: (), flip_y: () }
//     }
// }

impl Sprite {
    pub fn new(texture: Texture2D) -> Self {
        Self {
            texture: texture,
            pivot: Pivot::Center,
            custom_size: None,
        }
    }

    pub fn set_size(&mut self, x: f32, y: f32) {
        self.custom_size = Some(vec2(x, y));
    }

    pub fn world_size(&self) -> Vec2 {
        let size = self
            .custom_size
            .unwrap_or(vec2(self.texture.width(), self.texture.height()));
        vec2(size.x * PIXELS_PER_UNIT, size.y * PIXELS_PER_UNIT)
    }

    pub fn draw_default(&self) {
        let model_size = self.world_size();
        let offset = self.pivot.offset(model_size);

        draw_texture_ex(
            &self.texture,
            -1.0 * offset.x,
            -1.0 * offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(model_size),
                rotation: 0.0,
                ..Default::default()
            },
        );
    }

    pub fn draw(&self, transform: &Transform) {
        let model_size = self.world_size() * transform.scale;
        let offset = self.pivot.offset(model_size);

        draw_texture_ex(
            &self.texture,
            transform.pos.x - offset.x,
            transform.pos.y - offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(model_size),
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}

pub struct Pawn {
    transform: Transform,
    sprite: Sprite,
}

impl Pawn {
    pub fn new(position: Vec2, sprite: Sprite) -> Self {
        Pawn {
            transform: Transform {
                pos: position,
                ..Default::default()
            },
            sprite: sprite,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.transform.pos.x = x;
        self.transform.pos.y = y;
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.transform.scale.x = x;
        self.transform.scale.y = y;
    }

    fn collider(&self) -> Collider {
        // Collider::new(self.transform.pos, self.sprite.world_size())
        Collider::new_from_pawn(&self)
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        let collider = self.collider();
        let res = collider.contains_point(point);
        if res {
            println!("Collision at {:?}", point);
            println!("{:?}", collider);
            collider.draw_debug();
        }
        res
    }

    pub fn draw(&self) {
        // let translation = self.transform.pos;
        // draw_rectangle(translation.x, translation.y, 100.0, 100.0, RED);
        self.sprite.draw(&self.transform);
    }
}

#[derive(Debug, Clone)]
pub struct Transform {
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

#[derive(Debug, Clone)]
struct Collider {
    pub min: Vec2,
    pub max: Vec2,
}
impl Collider {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            min: Vec2::new(pos.x - size.x / 2.0, pos.y - size.y / 2.0),
            max: Vec2::new(pos.x + size.x / 2.0, pos.y + size.y / 2.0),
        }
    }

    pub fn new_from_pawn(pawn: &Pawn) -> Self {
        let pos = pawn.transform.pos;
        let scale = pawn.transform.scale;
        let size = pawn.sprite.world_size();
        let scaled_size = size / 2.0 * scale;
        Self {
            min: Vec2::new(pos.x - scaled_size.x, pos.y - scaled_size.y),
            max: Vec2::new(pos.x + scaled_size.x, pos.y + scaled_size.y),
        }
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        self.min.x <= point.x
            && point.x <= self.max.x
            && self.min.y <= point.y
            && point.y <= self.max.y
    }
    
    pub fn draw_debug(&self) {
        let w = (self.max.x - self.min.x).abs() * 0.05;
        let h = (self.max.y - self.min.y).abs() * 0.05;
        let pos = vec2(self.min.x - w, self.min.y - h);

        let w = (self.max.x - self.min.x).abs() * 1.1;
        let h = (self.max.y - self.min.y).abs() * 1.1;

        draw_rectangle(pos.x, pos.y, w, h, YELLOW);
    }
}
