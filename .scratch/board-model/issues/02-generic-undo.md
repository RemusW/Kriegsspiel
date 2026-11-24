# Undo written once, generic over board types

Status: resolved

## What

`Edit::between` and `Edit::apply` in `crates/server/src/model.rs` repeat the same diff and
merge logic for units and for maps. Pull it into a trait that each board type implements
(its id, and a per-field merge written with no `..`, as `merge_fields` and `merge_map` are
now), and make the diff, the apply and `UnitChange`/`MapChange` generic over it. Adding a
board type should then mean implementing the trait and adding one line to `Edit`.

Pure refactor: no behaviour or wire change.

## Why

Features, markers and line features are coming (issues 10 to 12). Without this, each one
copies about sixty lines of undo code.

## Notes

Maps currently remember their list position (`MapChange::at`) so an undone removal goes back
at the same height. Issue 06 replaces that with a `layer` field; until then the trait needs a
way for maps to keep it, or this issue can wait for 06. Pick whichever keeps the diff smaller
and say so in the PR.

## Acceptance

- All existing `model.rs` tests pass unchanged.
- No per-type copies of the diff or apply loops remain.
