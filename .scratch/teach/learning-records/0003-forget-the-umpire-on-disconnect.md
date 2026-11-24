# The umpire entry is dropped on disconnect, and why that is temporary

2026-11-23. Lesson 4 argued that nothing in the server ever frees a departed umpire's
`Doc::umpires` entry. The owner read it and came back with the fix as a directive: "for undos,
Doc umpires undo list should be deleted when an umpire disconnects." Implemented the same
session. `Doc::forget_umpire` removes the entry; `client()` now calls it after a new
`serve_client` that holds the loop, so the five exits that used to leak each meet in one
place. A `Drop` guard was rejected: `tokio::sync::RwLock` cannot be locked from `Drop`, and
`blocking_write` panics inside the runtime.

**Then the owner overruled me, and was right.** I argued the removal was sound only because
`UmpireId` comes from `fetch_add` per connection, so a departed id is unreachable, and that
identity would invert that and turn the same line into silent data loss &mdash; therefore it
should become a grace period. The owner's reply: "even with umpire identification. i want the
behavior to be that umpire's lose their undo history on a connection loss." Checking the
codebase settled it in their favour. `pack` takes only the board, the turns and the images,
and `Game::load` clears every history, so undo is already session-scoped and absent from a
save. Keeping it across a reconnect would have made undo *more* durable than the document it
describes. Recorded as ADR 0018.

**What that changes structurally, and is the better thing to teach.** `forget_umpire` is
garbage collection today and becomes the mechanism that *enforces* the rule the moment an id
can come back &mdash; same line, load-bearing for the opposite reason. The decision also
shrinks the identity work: identity no longer owes undo anything, so it is only about who may
do what. And it relocates the real cost, which is not lost work but invisibility:
`Step::Empty` is `Effect::None`, so Ctrl+Z after a reconnect sends nothing at all and the key
looks broken. A snapshot should carry whether undo and redo are available.

**Implications for teaching.** This is the first time a lesson produced a code change, which
makes it the moment to teach *preconditions* as a first-class idea rather than a footnote. The
transferable shape: a correctness argument that rests on an invariant held somewhere else in
the system, where the invariant is scheduled to change. Worth reaching for again when
presence arrives, since presence has the same shape (transient state keyed by an identity that
does not exist yet). Also: the owner acts on lessons, so from here on a lesson that names a
defect should expect to be read as a work item, and should say plainly what it would cost to
fix and what the fix would depend on.

**Recorded in the repo:** `CLAUDE.md` open questions (identity), the undo/redo behaviour
bullet, and the progress log. Related: [[0002-deltas-rejected]].
