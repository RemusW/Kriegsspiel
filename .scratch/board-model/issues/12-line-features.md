# Line features

Status: ready-for-agent
Blocked by: 02, 04

## What

- `LineFeature { id, kind: LineFeatureKind, points: Vec<[f32; 2]>, side: Option<Side> }` in
  `Board::line_features`. `points` merges as one field: two umpires editing the same line,
  the later edit wins.
- `LineFeatureKind`, a closed list: Fence, Wall, Hedge, Barricade, Trench.
- Commands to lay one down, replace its path, and remove it; all undoable, refusing
  non-finite points and paths of fewer than two points.
- Client: a tool to draw one point by point, and a way to select one and move its points.
  Drawn flat on the table above every map; where two cross, the later one is on top.

## Why

Umpires lay down and remove fences, barricades and trenches during a game, so they are board
state, not part of baked terrain. A line has no single position, so it is not a piece.

## Acceptance

- Draw a trench, extend it, undo the extension.
- A line drawn over two overlapping maps shows above both.
