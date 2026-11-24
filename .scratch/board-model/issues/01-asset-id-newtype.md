# AssetId newtype

Status: resolved

## What

Introduce `AssetId(String)` for the SHA-256 hex id of a stored image, and use it wherever an
asset id is held: `MapImage::asset`, `Pawn::image`, the `AddMap` and `SetUnitImage`
commands, `AssetStore`'s lookups, `Board::images()` and `save.rs`. It exports to TypeScript
as `string`, so the wire format does not change.

## Why

Both fields are bare `String`s today, so nothing in the types says they must name a stored
image. One type gives the store, the board and the save code a single thing to agree on.

## Acceptance

- No `String` field or parameter holds an asset id in `crates/server`.
- Generated bindings unchanged apart from any type alias; `npm run build` passes.
- `cargo test -p kriegsspiel-server` passes.
