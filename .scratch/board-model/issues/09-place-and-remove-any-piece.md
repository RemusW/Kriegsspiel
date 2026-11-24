# place and remove for any piece

Status: ready-for-agent
Blocked by: 03

## What

- `place` addresses pieces by id, whatever list holds them, so a drag over a mixed
  selection is one command and one undo step. A field the piece's type does not have is
  refused.
- New `remove { ids }`: takes pieces off the board, one undo step. Bind it to `Delete` as a
  press action.
- `Board::pieces()` iterates every piece; the client's selection holds ids of any piece and
  is pruned against all of them.

Only units exist when this lands, so the mixed case is exercised fully by 10 and 11, but the
command shape is settled here.

## Acceptance

- Delete removes the selection; undo brings it back.
- An unknown id in `place` or `remove` is skipped, as today.
