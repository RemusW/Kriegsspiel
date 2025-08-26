use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(ActionManager::default());
    app.add_systems(Update, handle_undo_redo);
}

pub trait Action: Send + Sync {
    fn execute(&mut self, commands: &mut Commands);
    fn undo(&mut self, commands: &mut Commands);
}


#[derive(Component)]
pub struct Movable;

#[derive(Debug)]
pub struct MoveAction {
    pub entity: Entity,
    pub from: Transform,
    pub to: Transform,
}

impl Action for MoveAction {
    fn execute(&mut self, commands: &mut Commands) {
        let entity = self.entity;
        let to = self.to.clone();
        commands.queue(move |world: &mut World| {
            if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                *transform = to;
            }
        });
        println!("move action: from {:?} to {:?}", self.from, self.to);
    }

    fn undo(&mut self, commands: &mut Commands) {
        let entity = self.entity;
        let from = self.from.clone();
        commands.queue(move |world: &mut World| {
            if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                *transform = from;
            }
        });
        println!("undo action: from {:?} to {:?}", self.from.translation, self.to.translation);
    }
}

#[derive(Resource, Default)]
pub struct ActionManager {
    undo_stack: Vec<Box<dyn Action>>,
    redo_stack: Vec<Box<dyn Action>>,
}

impl ActionManager {
    pub fn execute(&mut self, mut action: Box<dyn Action>, commands: &mut Commands) {
        action.execute(commands);
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    fn undo(&mut self, commands: &mut Commands) {
        if let Some(mut action) = self.undo_stack.pop() {
            action.undo(commands);
            self.redo_stack.push(action);
        }
    }

    fn redo(&mut self, commands: &mut Commands) {
        if let Some(mut action) = self.redo_stack.pop() {
            action.execute(commands);
            self.undo_stack.push(action);
        }
    }
}

fn handle_undo_redo(
    input: Res<ButtonInput<KeyCode>>,
    mut action_manager: ResMut<ActionManager>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::KeyZ) {
        action_manager.undo(&mut commands);
    }
    if input.just_pressed(KeyCode::KeyY) {
        action_manager.redo(&mut commands);
    }
}
