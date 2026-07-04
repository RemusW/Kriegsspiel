use uuid::Uuid;

use crate::{
    PawnManager, World,
    sprite::{Pawn, Transform},
};

#[derive(Debug)]
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

    fn label(&self) {
        match self {
            UnReCommand::Move(cmd) => println!("UnReCommand::Move"),
        }
    }
}

#[derive(Debug)]
pub struct MoveCommand {
    // pawn_uid: Uuid,
    // from: Transform,
    // dest: Transform,
    saved_pawns: Vec<(Uuid, Transform, Transform)>,
}

impl MoveCommand {
    fn new(uid: Uuid, from: Transform, dest: Transform) -> Self {
        Self {
            saved_pawns: vec![(uid, from, dest)],
        }
    }

    fn batch(moves: Vec<(Uuid, Transform, Transform)>) -> Self {
        Self { saved_pawns: moves }
    }

    fn execute(&self, pawn_manager: &mut PawnManager) {
        for (pawn_uid, _, dest) in self.saved_pawns.iter() {
            let pawn = pawn_manager.pawn_map.get_mut(&pawn_uid);
            if let Some(pawn) = pawn {
                pawn.set_transform(dest.clone());
            }
        }
    }

    fn undo(&self, pawn_manager: &mut PawnManager) {
        for (pawn_uid, from, _) in self.saved_pawns.iter() {
            let pawn = pawn_manager.pawn_map.get_mut(&pawn_uid);
            if let Some(pawn) = pawn {
                pawn.set_transform(from.clone());
            }
        }
    }
}

#[derive(Debug, Default)]
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

    pub fn execute(&mut self, cmd: UnReCommand, world: &mut World) {
        cmd.execute(world);
        self.history.push(cmd);
        self.redo.clear();
    }

    pub fn undo(&mut self, world: &mut World) {
        let cmd = self.history.pop();
        if let Some(cmd) = cmd {
            cmd.undo(world);
            self.redo.push(cmd);
        }
    }

    pub fn redo(&mut self, world: &mut World) {
        let cmd = self.redo.pop();
        if let Some(cmd) = cmd {
            cmd.execute(world);
            self.history.push(cmd);
        }
    }

    pub fn transform_pawn(moves: Vec<(Uuid, Transform, Transform)>) -> MoveCommand {
        // MoveCommand::new(pawn.get_uid(), from, pawn.transform.clone())
        MoveCommand::batch(moves)
        // self.execute(UnReCommand::Move(cmd), world);
    }
}
