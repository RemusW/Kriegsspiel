use uuid::Uuid;

use crate::{
    PawnManager, World,
    sprite::{Pawn, Transform},
};

pub enum UnReCommand {
    Move(MoveCommand),
}

impl UnReCommand {
    fn execute(&self, world: &mut World) {
        match self {
            UnReCommand::Move(cmd) => cmd.execute(&mut world.pawn_manager),
        }
    }

    fn undo(&self, world: &mut World) {
        match self {
            UnReCommand::Move(cmd) => cmd.undo(&mut world.pawn_manager),
        }
    }
}

pub struct MoveCommand {
    pawn_uid: Uuid,
    from: Transform,
    dest: Transform,
}

impl MoveCommand {
    fn new(uid: Uuid, from: Transform, dest: Transform) -> Self {
        Self {
            pawn_uid: uid,
            from,
            dest,
        }
    }

    fn execute(&self, pawn_manager: &mut PawnManager) {
        let pawn = pawn_manager.pawn_map.get_mut(&self.pawn_uid);
        if let Some(pawn) = pawn {
            pawn.set_transform(self.dest.clone());
        }
    }

    fn undo(&self, pawn_manager: &mut PawnManager) {
        let pawn = pawn_manager.pawn_map.get_mut(&self.pawn_uid);
        if let Some(pawn) = pawn {
            pawn.set_transform(self.from.clone());
        }
    }
}

pub struct CommandManager {
    history: Vec<UnReCommand>,
    redo: Vec<UnReCommand>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            history: Vec::default(),
            redo: Vec::default(),
        }
    }

    fn execute(&mut self, cmd: UnReCommand, world: &mut World) {
        cmd.execute(world);
        self.history.push(cmd);
        self.redo.clear();
    }

    fn undo(&mut self, world: &mut World) {
        let cmd = self.history.pop();
        if let Some(cmd) = cmd {
            cmd.undo(world);
            self.redo.push(cmd);
        }
    }

    fn redo(&mut self, world: &mut World) {
        let cmd = self.redo.pop();
        if let Some(cmd) = cmd {
            cmd.execute(world);
            self.history.push(cmd);
        }
    }

    pub fn transform_pawn(&mut self, world: &mut World, pawn: &Pawn, dest: Transform) {
        let cmd = MoveCommand::new(pawn.get_uid(), pawn.transform.clone(), dest);
        self.execute(UnReCommand::Move(cmd), world);
    }
}
