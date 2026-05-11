use crate::sprite::Transform;


trait Command {
    fn execute(&self);
    fn undo(&self);
}

struct MoveCommand {
    pawn_uid: u32,
    original: Transform,
    dest: Transform,
}

impl Command for MoveCommand {
    fn execute(&self) {
        
    }

    fn undo(&self) {
        
    }
}