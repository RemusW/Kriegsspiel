# Markers

Status: ready-for-agent
Blocked by: 02, 09

## What

- `Marker { id, kind: MarkerKind, pose, radius, side: Option<Side> }` in `Board::markers`.
- `MarkerKind`, a closed list: Smoke and Fire to start.
- A command to set one down and to change its radius; moved and removed by `place` and
  `remove` like any piece.
- Client: drawn as a disc on the table for now; particles are a later drawing change, not a
  model change.

## Why

Smoke is board state: it lasts across turns, is undoable and is saved. Cannon fire is an
event and does not belong here (see `GLOSSARY.md`).

## Acceptance

- Set down smoke, grow its radius, undo both.
