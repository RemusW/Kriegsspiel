# World units in ground metres

Status: ready-for-agent
Blocked by: 03

## What

Make one world unit one metre on the ground (ADR 0001).

- `Map` gains `metres_per_pixel`. `add_map` sets it to 1.0 and computes `size` from the
  image's pixel size; `NEW_MAP_WIDTH` goes away. New maps still go east of the others, with
  a gap in metres.
- A unit's footprint becomes a base size in metres per `UnitKind`, times `scale`. Choose
  plausible defaults (for example an infantry battalion in line about 150 m by 40 m) and
  keep them in one Rust table exported to the client, so the block geometry and hit testing
  read it rather than the fixed 4 by 2.
- `MIN_SCALE`/`MAX_SCALE` stay as limits on `scale`.
- Retune everything sized in the old arbitrary units: the blank table (100 by 70), camera
  speeds and limits in both modes, the pawn springs' lift and drop, and the outline width
  only if it is in world units.
- A format 1 save gets `metres_per_pixel` worked out from its old 100-wide map, so it opens
  looking as it did, at its old scale.

## Acceptance

- A 4000 px wide image laid down is 4000 m wide.
- Units, camera and springs feel the same at the new scale as before (check in the app).
- Format 1 sample save loads with its map at the size it had.
