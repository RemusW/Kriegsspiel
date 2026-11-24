# Map layers

Status: ready-for-agent
Blocked by: 03

## What

- `Map` gains `layer: f64`; maps are drawn in layer order, not list order. A new map goes
  above every other. f64 so a layer can always be put between two neighbours.
- Remove `MapChange::at` and the clamped re-insert: undoing a removal now restores the layer
  like any other field.
- Commands to bring a map forward or send it back one place, and to the top or bottom, as
  ordinary undoable edits merged per field.
- A format 1 save's single map gets layer 0.

## Why

A reorder that changes no value is an empty edit today, so it could not be undone. A field
merges like every other.

## Acceptance

- Bring forward, then undo, restores the order.
- Two umpires reordering different maps both keep their change.
