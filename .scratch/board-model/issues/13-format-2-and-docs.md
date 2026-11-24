# Close out save format 2 and the docs

Status: ready-for-agent
Blocked by: 01, 03, 04, 06, 08

## What

- Check that save format 2 is the shape the spec describes and that a format 1 save loads
  into it with every new field defaulted as the issues above say. Commit format 2 only after
  this; the issues before it may change format 2 freely because no format 2 file exists
  outside the owner's machine.
- Update CLAUDE.md to describe the code as it now is: Data model, Conventions (metres, unit
  footprints, layers, line features drawn above maps), the editor controls, the command list
  and the progress log. Remove anything no longer true.

Issues 09 to 12 may land after this one, as long as each adds its fields with defaults so
format 2 files written before it still load.

## Acceptance

- `crates/server/tests/sample_save.rs` covers a format 1 file and a format 2 file.
- CLAUDE.md names no type or field that no longer exists.
