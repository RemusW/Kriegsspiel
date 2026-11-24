# Board::units and Pose

Status: ready-for-agent
Blocked by: 02

## What

- Rename `Board::pawns` to `units` (JSON `units`).
- Replace `Pawn` with `Pose { pos, rot }`. `Unit` holds `pose`, and `scale` and `image` move
  onto `Unit` itself. Keep the JSON flat with `#[serde(flatten)]`.
- Rename `MapImage` to `Map` and give it a `pose`, so maps gain `rot` (0 for every map laid
  so far). Nothing turns a map yet; the client draws `rot` anyway.
- Update the client: generated bindings, `PendingEdits`, `PawnLayer` and everything that
  reads `board.pawns`. Client names that say "pawn" for a unit become "unit"; `PawnLayer`
  may keep its name if it will draw every piece (decide and say which).
- Save format 2 reads and writes the new shape; format 1 saves still load.

## Why

`pawns` holds only units, and "pawn" is not a domain term (see `GLOSSARY.md`: **Unit**,
**Piece**). The placement shared by every piece is a pose; scale and image are a unit's own.

## Acceptance

- `grep -ri pawn crates/server/src` finds nothing but deliberate leftovers explained in the PR.
- Format 1 sample save still loads (`crates/server/tests/sample_save.rs`).
- `cargo test -p kriegsspiel-server` and `npm run build` pass; the app still drags, turns and
  resizes units.
