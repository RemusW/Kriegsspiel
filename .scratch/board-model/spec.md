# Board model: maps, pieces and line features

Settled in a design session on 2026-11-21. Terms are defined in `GLOSSARY.md`; the world-unit
decision is `docs/adr/0001-world-units-are-ground-metres.md`.

## Goal

Replace the board's two ad hoc lists (`pawns` holding only units, `maps` with its own
placement fields) with one consistent model that can grow to features, markers and line
features, in real ground metres, without each new type copying the undo code.

## The model

```rust
Board { maps, units, features, markers, line_features, next_id } // one id counter for all
Pose  { pos: [f32; 2] /* metres */, rot: f32 }

Map         { id, asset: AssetId, pose, layer: f64, metres_per_pixel, size /* server-set */ }
Unit        { id, kind, side, formation, pose, scale, image: Option<AssetId> }
Feature     { id, kind: FeatureKind, pose, scale, side: Option<Side>, model: Option<AssetId> /* later */ }
Marker      { id, kind: MarkerKind, pose, radius, side: Option<Side> }
LineFeature { id, kind: LineFeatureKind, points: Vec<[f32; 2]>, side: Option<Side> }
```

- **One list per type.** No list holds another's items; `Board::pieces()` iterates units,
  features and markers together where something needs all of them.
- **World unit = 1 m on the ground.** A map's `size` is its pixel size times
  `metres_per_pixel`, computed by the server. A new map starts at 1 m/px; the umpire
  calibrates it against the map's scale bar. The image's DPI is never read.
- **A unit's footprint** is base(kind, formation), in metres, times `scale`. `formation` is
  what rules read; `scale` is the umpire's free adjustment. Each unit kind allows only some
  formations.
- **A feature's footprint** is base(kind), in metres, times `scale`.
- **Side** on features, markers and line features is Red, Blue or none (`Option<Side>`), so
  per-side filtering can hide them. Units always have a side. `Side` stays a closed
  Blue/Red enum until scenarios define their own sides.
- **Layer** orders maps only: the higher one is seen where two overlap. Line features always
  draw above every map; where two cross, the later one is on top.
- **Closed lists:** `UnitKind`, `Formation`, `FeatureKind`, `MarkerKind`, `LineFeatureKind`.
  Custom feature models come later as an optional asset.
- **Events** such as cannon fire are not board state.

## Commands

- `place` and `remove` take ids of any piece (unit, feature, marker), so one drag or one
  delete over a mixed selection is one undo step. Fields a type lacks are refused.
- Maps are changed only in a separate map editing mode: move and turn, `setMapScale`
  (calibrate), and change layer.
- Line features have their own commands to lay one down, replace its path, and remove it.

## Undo

Written once, generic over a trait every board type implements (an id, and a per-field merge
with no `..` so a new field fails to compile until handled). A line feature's path merges as
one field.

## Saves

All of this lands in save format 2 before it is committed; format 1 saves keep loading.
