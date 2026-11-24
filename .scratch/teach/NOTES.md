# Teaching notes

## Preferences

- Workspace lives in `.scratch/teach/` (gitignored) so the repository root stays bare.
  Decided 2026-11-21.
- Graphics programmer. Skip graphics and three.js basics entirely. Analogies to frame loops,
  keyframe interpolation and double buffering are welcome where they are accurate, not
  decorative.
- Wants to read the code unaided and explain it out loud. So every lesson should quote the
  actual file and line, and end with something to say aloud, not just recognise.
- Asked for three things at once (read the code / fundamentals / interview-ready). Treat
  "interview-ready" as the test of the other two: if it can be said aloud from memory, it
  is learned.

## Working notes

- Self-assessed weak spots, session 1: why WebSocket at all, the `tokio::select!` loop,
  optimistic updates. That is the whole spine of the model, in dependency order. Teach it in
  that order: transport, then fan-out, then the client's local preview.
- Planned sequence after lesson 1:
  - 0002 - the per-connection loop: `tokio::select!`, the watch channel, `Effect`, why each
    connection renders its own snapshot.
  - 0003 - optimistic edits: `PendingEdits`, "what is on screen is board + edits in flight",
    why a refusal still sends a board. Gambetta as primary source.
  - 0004 - reconnection and identity: `next_umpire`, what is lost on reconnect, what a real
    `UmpireId` costs.
  - 0005 - the snapshot's cost, and why presence cannot use it. Leads into step 5.
- Spacing: open each lesson from 0002 on with two recall questions about the previous one,
  before any new material. Done in 0002 (why HTTP cannot push; what ordering is spent on).
- 0002 shipped 2026-11-22. It teaches cancel safety as new material rather than background,
  per LR-0001. The worked example &mdash; axum documents no cancel safety for `recv`, so read
  three lines of its source and apply tokio's rule &mdash; is the bit most worth checking
  retention on. Open 0003 by asking for the two reasons the watch channel carries no data,
  and for why `send_view` drops the lock before writing.
- Settled 2026-11-23: per-unit deltas rejected, `CLAUDE.md` amended. See LR-0002. Do not
  teach deltas as a planned future state. `docs/adr/0021` still carries a softer version of
  the line ("left until measurements call for it") &mdash; offered to amend it, owner has not
  said yet.
- 0003 shipped 2026-11-23. Its sharpest point is that this project does optimistic updates,
  *not* Gambetta's client-side prediction, and the two reasons it can skip sequence numbers
  (absolute commands; no simulation). Check that one survives into lesson 4's warm-up.
- Three loose ends offered at the end of 0003, any of which could become a lesson:
  `Step::Partial` and telling an umpire *why* a command was refused (the one genuinely
  unfinished part of the model); whether `PendingEdits` should cover spawns and deletions,
  which it structurally cannot today; and why pending edits live in the engine rather than
  the Zustand store. That last one is the best React teaching opportunity in the whole
  codebase &mdash; per-frame state vs. render-triggering state &mdash; and 0019 already
  decided it. Consider it for 0005.
- 0004 shipped 2026-11-23, and **the owner acted on it the same day**: they read the leak
  section and directed the fix ("for undos, Doc umpires undo list should be deleted when an
  umpire disconnects"). Built it: `Doc::forget_umpire`, called from `client()` around a new
  `serve_client`. See learning record 0003. Two consequences for teaching. First, a lesson
  that names a defect here will be read as a work item, so say what the fix costs and what it
  rests on. Second, the lesson's framing &mdash; leak and identity as one piece of work
  &mdash; turned out to be wrong, and the owner said so: they want undo lost on a connection
  drop *even once identity exists*. The codebase backed them (undo is in no save and cleared
  by a load). Now ADR 0018. Lesson 4 carries a callout recording both the fix and my corrected
  argument; the leak claim in it is deliberately left standing, because the fix only makes
  sense after the argument.
- **Teaching note from that exchange:** the owner argues back against a recommendation and
  wins on the merits. So state recommendations as positions with their evidence, and name
  what would falsify them &mdash; do not soften them into options, and do not defend one once
  the code disagrees. Also worth noticing: the decision came from asking what undo *is* (a
  record of this session, not game state), not from weighing mechanisms. Teach the
  "what kind of state is this?" question explicitly &mdash; it is the same move that settled
  deltas in 0002.
- 0004 also makes a recommendation rather than leaving the security gap open: check `Origin`
  on upgrade first (cheap, closes the drive-by case, is what RFC 6455 §10.2 is for), then a
  token for identity continuity. Deliberately did not hand over a token scheme &mdash; it
  interacts with the relay's join-by-code, which is undesigned, so it is better talked
  through. If the owner asks, the live sub-question is where the token travels: query string
  (lands in logs), `Sec-WebSocket-Protocol` (awkward but a real header), or a first message
  (leaves an unidentified window).
- The server thread ended with 0004. 0005 shipped 2026-11-23: *where state lives* &mdash; why
  `PendingEdits` is not in Zustand. Thesis: a store is a subscription, not storage, so the
  change rate of a value is the render rate of everything reading it. The strongest evidence
  turned out to be in the owner's own code: `sameTurns` and the kept-selection-array trick in
  `showView` are hand-written memoisation that exists *only* because those values are in the
  store, so the cost of membership is visible on the page. react.dev's "React does not touch
  the DOM if the rendering result is the same as last time" sharpened the claim from "it would
  be slow" to "the commit phase is empty by construction" &mdash; worth checking primary
  sources even when the conclusion was never in doubt.
- The interview answer 0005 is built to produce: `PendingEdits` : client store :: presence :
  `Doc`. One principle applied at both ends of the wire. This is the first lesson that closes
  a loop back to the server thread, and it landed naturally rather than being forced.
- 0006 shipped 2026-11-23: *smoothing what arrives late*. Two findings. (1) `spring.ts` is a
  complete implementation of the non-buffering smoothing strategy, written for physicality,
  with the network benefit incidental &mdash; the lesson's job was renaming, not teaching.
  (2) `CLAUDE.md`'s presence rule "drop outdated ones instead of queueing them" *forbids*
  entity interpolation, since interpolation is a queue. So the smoothing decision was already
  made in a line written about bandwidth. See learning record 0004 for the unifying thread
  ("latest state wins"), which is now the best single answer the series has produced.
- **The highest-leverage move with this owner**, found in 0006: take something they already
  built from graphics instinct and give it its networking name and literature. They trust the
  code; what they lack is the vocabulary and the trade-off. Look for more of these
  deliberately. Candidates: the fixed-step accumulator in `PawnLayer.update` (vs. network
  tick rate), and `hitTestUnits` picking against units as drawn (vs. server-side hit
  validation in shooters).
- 0006 section 06 derives a ~63 ms spring settle time from the `FOLLOW` constants against a
  50 ms presence interval, and is labelled an estimate per the repo's own convention. It
  contains a falsifiable prediction (10 Hz presence should visibly stop-start). If the owner
  ever measures it, update the lesson and say so.
- 0007 shipped 2026-11-23: *the shape of presence* &mdash; the first design lesson, and the
  last one the repo has enough written design to support. Centrepiece: **presence is the
  unsent prefix of an edit**, carrying `Vec<Placement>`, the same type a `place` command
  carries, which means `previewOf` / `PendingEdits` / `PawnLayer` already render it. Also: it
  must NOT use the `changed` doorbell (that would serialise a whole board per tick), and the
  vanishing-sender problem is already solved by `client()`'s single exit point, built earlier
  this session for undo &mdash; the lesson gets to collect that.
- **Checked a claim and was wrong about it**, worth remembering: I was about to argue
  "broadcast queues, watch does not". The tokio docs say otherwise &mdash; broadcast drops
  too, but a lagger resumes from the *oldest retained value*, so it replays stale frames. The
  real distinction is sequence-preserving vs. latest-only. Fetching the doc changed the
  argument, not just the citation. Second time this session that verifying improved a claim
  (the other was react.dev's "does not touch the DOM"). Keep doing it, and keep saying in the
  lesson which claims are unverified &mdash; 0007 flags `send_modify` and the borrow-guard
  pattern explicitly.
- **The series has caught up with the code.** 0007's Next section offers three directions
  rather than assuming one: build the 20-line `lifted` change; a lesson on why `PendingEdits`
  cannot cover spawns and deletions (the best remaining hole); or leave the repo for
  backpressure/heartbeats/long-haul WebSocket behaviour, which RESOURCES.md still lists as a
  gap and which 0007 sharpened into a named one (structural cleanup cannot see a half-open
  connection). Let the owner pick; do not assume the next lesson.
- Standing offers still unanswered: `docs/adr/0021`'s stale delta line; the token design as an
  ADR; the `InviteId`/`SessionId` split; and now moving CLAUDE.md's presence entry out of open
  questions into Decisions if 0007's design is accepted.
- Three live offers left open across 0004 and 0005, all declined-so-far rather than refused:
  `docs/adr/0021`'s stale delta line; the token design (talked through in conversation
  2026-11-23 but not written up as an ADR); and the `InviteId`/`SessionId` split, which is
  the concrete first step if identity gets built.
- 2026-11-24: **the owner wants to write code, not read it.** They asked where to start on
  the lift, I built the server half, and they stopped me and had it reverted, then asked for
  something to implement themselves. Lesson learned: when they say "let's build", the default
  is a brief and a feedback loop, not an implementation. Check which they want before writing
  code in their repo.
- 0008 shipped 2026-11-24: *build it: the presence channel* &mdash; the first exercise lesson.
  Six failing tests in `exercises/0001-presence-tests.rs` are the feedback loop; the lesson is
  seven tasks with hints behind `details.hint` (new reusable component in `lesson.css`). No
  worked solution exists anywhere in the workspace, deliberately &mdash; say so when asked.
  Scoped to held pieces, no cursors, because a drag has two events and so needs no throttle.
  Two corrections to 0007 recorded in it: the "twenty-line version" was only the drawing half,
  and the wire should carry the union of others' holdings rather than a map keyed by umpire,
  which keeps `UmpireId` out of the protocol entirely.
- **Unverified in 0008:** the six tests were written against an implementation that compiled,
  but the run was interrupted, so none of them has been seen to pass. Two
  (`an_unchanged_holding_wakes_nobody`, `an_umpire_holding_nothing_has_no_entry`) are new and
  desk-checked only. If the owner fights a test, suspect the test.
- The exercise format worked out well for this codebase because `cargo test` is a real loop.
  The client has no test setup at all (no vitest), so a TypeScript exercise cannot have an
  automatic feedback loop without a dependency decision that is the owner's to make. Keep
  exercises on the Rust side until that changes, or accept two browser windows as the loop.
- The repository's own `GLOSSARY.md` holds domain language (board, piece, umpire). The
  workspace `GLOSSARY.md` holds networking language only. Keep them from drifting.
