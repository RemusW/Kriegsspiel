use macroquad::prelude::*;
use uuid::Uuid;

use crate::{
    World,
    sprite::{Pawn, Transform},
};

#[derive(Default)]
pub struct EditorState {
    pub tool_mode: ToolMode,
    pub selection: SelectionTool,
}

impl EditorState {
    pub fn update(&mut self, world: &mut World) {
        match self.tool_mode {
            ToolMode::Spawn => {
                // spawn_pawn(&mut world, &cavalry);
            }
            ToolMode::Move => {
                self.selection.process_pick(world);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum ToolMode {
    #[default]
    Spawn,
    Move,
}

impl ToolMode {
    pub fn label(&self) -> &str {
        match self {
            ToolMode::Spawn => "Select",
            ToolMode::Move => "Move",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SelectionTool {
    selections: Vec<Uuid>,

    pre_picked_center: Vec2,
    pre_transforms: Vec<Transform>,
}

impl SelectionTool {
    fn process_pick(&mut self, world: &mut World) {
        if is_mouse_button_pressed(MouseButton::Left) {
            // save transform state of mouse and picked objects
            self.get_selected_pawns(world);
            self.pre_picked_center = mouse_position().into();
            self.save_pre_transform(world);
        }
        if is_mouse_button_down(MouseButton::Left) {
            // update mouse center and move all selected objects relative to anchor
            let selected_pawns = world.pawn_manager.get_pawns_from_uid_mut(&self.selections);
            let screen_pos: Vec2 = vec2(
                self.pre_picked_center.x + mouse_position().0,
                self.pre_picked_center.y + mouse_position().1,
            );
            let new_pos = world.camera.screen_to_world(screen_pos);
            for pawn in selected_pawns {
                let pos = pawn.transform.pos.clone();
                // let new_pos = (pos.x + delta.x, pos.y + delta.y);
                pawn.set_position(new_pos.x, new_pos.y);
                println!("{:?} \n {:?}", pos, new_pos);
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            // Create the TransformCommand and record it
        }
    }

    fn get_selected_pawns(&mut self, world: &mut World) {
        for (_, pawn) in world.pawn_manager.pawn_map.iter_mut() {
            let contains =
                pawn.contains_point(world.camera.screen_to_world(mouse_position().into()));
            if contains {
                self.selections.push(pawn.get_uid());
            }
        }
    }

    fn save_pre_transform(&mut self, world: &mut World) {
        // self.selections.iter().map(f)
        for uid in &self.selections {
            if let Some(pawn) = world.pawn_manager.pawn_map.get(&uid) {
                self.pre_transforms.push(pawn.transform.clone());
            }
        }
    }
}
