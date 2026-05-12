use crate::{PawnUID, sprite::Transform};

trait Command {
    fn execute(&self);
    fn undo(&self);
}

pub enum GameCommand {
    Move(MoveCommand),
}

impl GameCommand {
    fn execute(&self) {
        match self {
            GameCommand::Move(cmd) => cmd.execute(),
        }
    }

    fn undo(&self) {
        match self {
            GameCommand::Move(cmd) => cmd.undo(),
        }
    }
}

struct MoveCommand {
    pawn_uid: u32,
    from: Transform,
    dest: Transform,
}

impl MoveCommand {
    fn new(uid: u32) -> Self {

    }
}

impl Command for MoveCommand {
    fn execute(&self) {
        
    }

    fn undo(&self) {
        
    }
}

struct CommandManager {
    history: Vec<GameCommand>,
    redo: Vec<GameCommand>,
}

impl CommandManager {
    fn execute(&mut self, cmd: GameCommand) {
        cmd.execute();
        self.history.push(cmd);
        self.redo.clear();
    }

    fn undo(&mut self) {
        let cmd = self.history.pop();
        if let Some(cmd) = cmd {
            cmd.undo();
            self.redo.push(cmd);
        }
    }

    fn redo(&mut self) {
        let cmd = self.redo.pop();
        if let Some(cmd) = cmd {
            cmd.execute();
            self.history.push(cmd);
        }
    }

    pub fn transform_pawn(&mut self, pawn_id: PawnUID, dest: Transform, ctx: &mut GameCommand) {
        let cmd = MoveCommand::new(pawn_id, dest, ctx);
        self.execute(GameCommand::Move(cmd), ctx);
    }
}