# Features

Status: ready-for-agent
Blocked by: 02, 04, 09

## What

- `Feature { id, kind: FeatureKind, pose, scale, side: Option<Side> }` in `Board::features`,
  with undo through the trait from 02.
- `FeatureKind`, a closed list of defaults (for example House, Church, Farm, Mill, Bridge,
  Emplacement), each with a base footprint in metres. Footprint = base times `scale`.
- A command to set one down, and the side set to Red, Blue or none.
- Client: a simple placeholder mesh per kind, picked, selected, dragged, turned, scaled and
  deleted like units through `place` and `remove`, and mixed into the same selection.
- Leave room for an optional custom model asset later; do not build model import here.

## Acceptance

- A box selection over a farm and a battalion turns both with Q/E in one undo step.
- A feature with no side and one with a side both survive a save and load.
