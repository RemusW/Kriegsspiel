# "Latest state wins" is the architecture, and it decides three separate questions

2026-11-23. Written while building lesson 0006. The owner has now met three apparently
unrelated design questions and found the same answer behind each:

- **Per-unit deltas** (0002, rejected): a delta needs every earlier delta, and the `watch`
  channel's coalescing deliberately will not promise that.
- **Replay on reconnect** (0004, deliberately absent): a queued `place` carries absolute
  values computed against a board that has since moved, so replaying it would overwrite newer
  work.
- **Entity interpolation** (0006, ruled out): it cannot function without retaining at least
  two timestamped samples, and `CLAUDE.md`'s presence rule says "drop outdated ones instead of
  queueing them".

One commitment underneath all three: *latest state wins, and history is not retained in
transit.* It is not written down anywhere in the repo as a principle. It is enforced in three
places by three mechanisms that were each chosen for local reasons &mdash; coalescing for
cheapness, absolute commands for idempotence, dropping for bandwidth.

**Why this is the most valuable thing the series has produced.** It converts a pile of
individual decisions into one sentence the owner can defend, which is exactly what the "explain
it in interviews" leg of the mission asked for. It is also predictive rather than descriptive:
any future technique that needs a sequence (lockstep, rollback, command replay, animation
playback from the wire, bandwidth-saving diffs of any kind) will collide with it, and that can
be known before designing. Use it as the first question asked of any new networking proposal
in this project: *does this need history that nothing promises to keep?*

**The second finding of 0006, smaller but more immediately useful.** `pawns/spring.ts` is a
complete implementation of the non-buffering smoothing strategy, built for physicality rather
than for networking, and the network benefit was incidental. So presence needs almost no
smoothing work: what it needs is a cursor layer keyed by umpire rather than unit id, and a
second input to `lifted` so a piece can be shown held by someone else. Both are identity
problems wearing rendering costumes, which puts them behind the token design.

**Teaching note.** This is the first lesson where the owner's existing expertise did most of
the work and my job was almost entirely renaming: showing that something they built is a known
technique with a known trade-off and a literature. That is probably the highest-leverage move
available with this owner, and it is worth looking for deliberately rather than waiting for it
&mdash; where else does graphics work they already trust have a networking name?

Related: [[0002-deltas-rejected]], [[0003-forget-the-umpire-on-disconnect]].
