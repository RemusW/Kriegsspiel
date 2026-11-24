# Formations

Status: ready-for-agent
Blocked by: 04

## What

- `Formation`, a closed enum: Line, Column, Square, Skirmish (infantry), Mounted, Dismounted
  (cavalry), Limbered, Unlimbered (artillery). Each `UnitKind` allows only some.
- `Unit` gains `formation`; `spawn` gives each kind a default.
- The base footprint table from 04 becomes per (kind, formation). Footprint = base times
  `scale`; the two fields merge independently, so one umpire changing formation and another
  changing scale both keep their change.
- `setFormation { units, formation }`, one undo step for the selection, refusing a
  formation a unit's kind does not allow.
- Client: a way to set the formation of the selection (toolbar or a bindable action), and
  blocks drawn at their formation's shape.
- A format 1 save's units get their kind's default formation.

## Acceptance

- Infantry in column is narrower and deeper than in line.
- Refused: Square for artillery.
- Concurrent formation and scale edits both survive.
