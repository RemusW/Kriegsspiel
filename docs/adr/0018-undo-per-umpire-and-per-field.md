# Undo is per umpire and per field

Each umpire has their own undo history, and undoing restores each field only where the board
still holds what the edit left there. An umpire's undo must not revert another umpire's work,
which whole-board snapshots cannot achieve.

An umpire's history lasts exactly as long as their connection: it is dropped when the
connection ends, and a reconnecting umpire starts with an empty stack even once identity lets
them present the same `UmpireId`. Undo is a record of what this umpire just did, not part of
the game, which is why a save does not carry it and loading one clears it; surviving a
reconnect would make it more durable than the document it describes. A stale stack is also
mostly unusable, since every edit another umpire has made since turns an undo into
`Step::Partial`. The cost is that a brief drop silently empties the stack, so a snapshot
should tell the client whether undo and redo are available rather than leaving Ctrl+Z to do
nothing.
