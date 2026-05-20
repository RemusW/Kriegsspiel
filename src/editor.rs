use macroquad::prelude::*;
use uuid::Uuid;

use crate::{World, sprite::Pawn};

#[derive(Default)]
pub struct EditorState {
    pub tool_mode: ToolMode,
    pub selection: SelectionTool,
}

impl EditorState {
    pub fn update(&self, world: &mut World) {
        match self.tool_mode {
            ToolMode::Spawn => {
                // spawn_pawn(&mut world, &cavalry);
            }
            ToolMode::Selection => {
                let mut selected_pawns: Vec<&mut Pawn> = Vec::new();
                for (_, pawn) in world.pawn_manager.pawn_map.iter_mut() {
                    let contains =
                        pawn.contains_point(world.camera.screen_to_world(mouse_position().into()));
                    if contains {
                        selected_pawns.push(pawn);
                    }
                }
                if is_mouse_button_down(MouseButton::Left) {
                    let delta = mouse_delta_position() * 2.0;
                    for pawn in selected_pawns {
                        let pos = pawn.transform.pos.clone();
                        let new_pos = (pos.x + delta.x, pos.y + delta.y);
                        pawn.set_position(new_pos.0, new_pos.1);
                        println!("{:?} \n {:?}", pos, new_pos);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum ToolMode {
    #[default]
    Spawn,
    Selection,
}

impl ToolMode {
    pub fn label(&self) -> &str {
        match self {
            ToolMode::Spawn => "Select",
            ToolMode::Selection => "Selection",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SelectionTool {
    selections: Vec<Uuid>,
    is_picking: bool,
    offset: Vec2,
}

impl SelectionTool {
    fn process_pick(&self) {
        if self.is_picking {}
    }
}
