# Map editing mode

Status: ready-for-agent
Blocked by: 03

## What

A separate tool in `TOOLS` for working on maps. In it, maps are picked and selected instead of
pieces; dragging moves the selected maps and the turn keys turn them, through the same
`Manipulation` as pieces, sent as one command on release. Map selection is separate from the
piece selection. Outside this mode a click never grabs a map.

Server: a command to set maps' poses, an ordinary undoable edit, validated like `place`.

The calibrate action (05) and layer buttons (06) belong in this mode's UI.

## Why

A map is the ground everything stands on; moving one by accident under a missed click on a
unit would be easy to do and hard to notice.

## Acceptance

- In map mode, drag and turn a map; one undo step puts it back.
- In the select tool, clicking an empty spot on a map still clears the selection.
