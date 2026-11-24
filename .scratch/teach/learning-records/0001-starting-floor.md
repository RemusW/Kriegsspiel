# Starting floor: the network model is read but not owned

Session 1, 2026-11-21. The owner can read the networking code and use its vocabulary
(`command`, `snapshot`, `Effect`, authoritative server) correctly in prose &mdash; CLAUDE.md
and the code comments are evidence &mdash; but says the *reasoning* underneath three things
is not solid: why a WebSocket rather than HTTP, the `tokio::select!` connection loop, and
optimistic updates via `PendingEdits`.

**Implications.** Do not re-teach what a command or a snapshot is; that floor is already
set. Teach the *why* beneath each mechanism, in dependency order (transport, then fan-out,
then the client's local preview), and treat "can say it out loud from memory" as the bar,
since the mission is partly interview readiness. Async Rust cannot be assumed: when lesson
0002 reaches `tokio::select!`, teach cancellation and cancel-safety as new material rather
than as background.

**Evidence of the gap, in the owner's own words:** picked all three of "why WebSocket at
all", "the socket loop" and "optimistic updates" when asked which felt least solid, and
gave the reason that much of this code was written with an agent's help.
