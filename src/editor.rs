use macroquad::{miniquad::CursorIcon::Move, prelude::*};
use uuid::Uuid;

use crate::{
    AppContext, World,
    asset::AssetStore,
    command::{CommandManager, MoveCommand, UnReCommand},
    sprite::{Pawn, Transform},
};

#[derive(Default)]
pub struct EditorState {
    pub tool_mode: ToolMode,
    pub selection: SelectionTool,
}

impl EditorState {
    pub fn update(&mut self, world: &mut World, command_manager: &mut CommandManager, asset_store: &AssetStore) {
        match self.tool_mode {
            ToolMode::Spawn => {
                // spawn_pawn(&mut world, &cavalry);
            }
            ToolMode::Move => {
                if let Some(cmd) = self.selection.process_pick(world, asset_store) {
                    command_manager.execute(cmd, world);
                }
            }
        }
        if is_key_down(KeyCode::LeftControl) {
            if !is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::Z) {
                println!("UNDOING");
                command_manager.undo(world);
            }
            if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::Z) {
                println!("REDOING");
                command_manager.redo(world);
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
    fn process_pick(&mut self, world: &mut World, asset_store: &AssetStore) -> Option<UnReCommand> {
        if is_mouse_button_pressed(MouseButton::Left) {
            // save transform state of mouse and picked objects
            self.update_selected_pawns(world, asset_store);
            if self.selections.is_empty() {
                self.pre_transforms.clear();
                println!("mouse pressed: clearing seleciton");
            } else {
                self.pre_picked_center = world.camera.screen_to_world(mouse_position().into());
                self.save_pre_transform(world);
            }
        }
        if is_mouse_button_down(MouseButton::Left) {
            // update mouse center and move all selected objects relative to anchor
            let mut selected_pawns = world.pawn_manager.get_pawns_from_uid_mut(&self.selections);
            let mouse_pos = world.camera.screen_to_world(mouse_position().into());
            let offset = mouse_pos - self.pre_picked_center;
            for (i, pawn) in selected_pawns.iter_mut().enumerate() {
                let pos = self.pre_transforms[i].pos + offset;
                pawn.set_position(pos.x, pos.y);
                // println!("{:?} \n {:?}\n", pos, offset);
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            // Create the TransformCommand and record it
            let selected_pawns = world.pawn_manager.get_pawns_from_uid_mut(&self.selections);
            let moves: Vec<(Uuid, Transform, Transform)> = selected_pawns
                .iter()
                .enumerate()
                .map(|(i, pawn)| {
                    (
                        pawn.get_uid(),
                        self.pre_transforms[i].clone(),
                        pawn.transform.clone(),
                    )
                })
                .collect();
            let move_cmd = CommandManager::transform_pawn(moves);
            self.pre_transforms.clear();
            return Some(UnReCommand::Move(move_cmd));
        }
        None
    }

    fn update_selected_pawns(&mut self, world: &mut World, asset_store: &AssetStore) {
        let mut cur_picked: Vec<Uuid> = Vec::new();
        for (_, pawn) in world.pawn_manager.pawn_map.iter_mut() {
            let contains = pawn.contains_point(
                world.camera.screen_to_world(mouse_position().into()),
                asset_store,
            );
            if contains {
                cur_picked.push(pawn.get_uid());
            }
        }

        if cur_picked.is_empty() {
            self.selections.clear();
            return;
        }

        // Additive select
        if is_key_down(KeyCode::LeftShift) {
            for uid in cur_picked.iter() {
                if !self.selections.contains(uid) {
                    self.selections.push(*uid);
                }
            }
            return;
        }

        // Single select
        let mut already_selected = false;
        for uid in cur_picked.iter() {
            if self.selections.contains(uid) {
                already_selected = true;
                break;
            }
        }
        if !already_selected {
            self.selections.clear();
            self.selections.append(&mut cur_picked);
        }
    }

    pub fn get_selected_pawns<'a>(&self, world: &'a World) -> Vec<&'a Pawn> {
        world
            .pawn_manager
            .get_pawns_from_uid(self.selections.clone())
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
