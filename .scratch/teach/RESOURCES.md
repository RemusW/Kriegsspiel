# Network model resources

## Knowledge

- [RFC 6455: The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455.html)
  The protocol itself. Section 1.1 states the motivation against HTTP polling in three
  sentences; 1.2 and 1.3 walk the Upgrade handshake; 1.5 describes it as "just a layer on
  top of TCP". Use for: anything about framing, the handshake, ordering, what a socket
  guarantees. Short and readable for an RFC.

- [RFC 6455 &sect;10, Security Considerations](https://www.rfc-editor.org/rfc/rfc6455.html#section-10)
  &sect;10.2 names this project's exact gap: a server "SHOULD verify the |Origin| field is an
  origin they expect" and otherwise answer 403, in order to "ensure that trusted browsers
  under the control of potentially malicious JavaScript cannot fake a WebSocket handshake" —
  while noting "the intent is not to prevent non-browsers from establishing connections".
  &sect;10.5 is one sentence: the protocol "doesn't prescribe any particular way that servers
  can authenticate clients during the WebSocket handshake". Use for: the Origin fix, and for
  why there is no standard auth pattern to copy.

- [MDN: WebSocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
  The browser-side API `ServerConnection.ts` is written against: `readyState`, the four
  events, and the note that the API has **no backpressure** ("when messages arrive faster
  than the application can process them, the application will either fill up the device's
  memory by buffering those messages, become unresponsive due to 100% CPU usage, or both").
  Use for: client lifecycle, reconnection, what `send` does when not open.

- [MDN: The WebSocket API guide index](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)
  Entry point to the client and server guides, and to `WebSocketStream` (the
  backpressure-aware alternative). Use for: orienting before reading the reference pages.

- [Tokio: `sync::watch`](https://docs.rs/tokio/latest/tokio/sync/watch/index.html)
  "A multi-producer, multi-consumer channel that only retains the last sent value." The
  mechanism behind `Game.changed`. Use for: why fan-out works the way it does, and why a
  slow connection cannot make the server queue up boards.

- [Tokio: `watch::Receiver::changed`](https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html#method.changed)
  States plainly that the method **is cancel safe**: "If you use it as a branch in
  tokio::select! and another branch completes first, then it is guaranteed that no values
  have been marked seen by this call to changed." Use for: justifying the `tokio::select!`
  in `client()`.

- [Tokio: `watch::Sender::send_replace`](https://docs.rs/tokio/latest/tokio/sync/watch/struct.Sender.html#method.send_replace)
  "This method permits sending values even when there are no receivers", unlike `send`,
  which "fails if the channel is closed, which is the case when every receiver has been
  dropped." Use for: why the server calls `send_replace` and not `send`.

- [Tokio: `sync::broadcast`](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
  The lagging section is the whole reason to read it. A receiver that falls behind gets
  `RecvError::Lagged` with the count it missed, which "does not close or disconnect the
  receiver" &mdash; its cursor moves to the *oldest retained value* and it carries on from
  there. So broadcast preserves the sequence (replaying stale frames to a lagger) where
  `watch` preserves only the latest. That is the real distinction behind CLAUDE.md's "drop
  outdated ones instead of queueing them", and the sloppy version of the argument ("broadcast
  queues") is wrong. Used in lesson 0007.

- [Tokio: `select!`](https://docs.rs/tokio/latest/tokio/macro.select.html)
  "Waits on multiple concurrent branches, returning when the first branch completes,
  cancelling the remaining branches." Use for: the shape of the per-connection loop, and the
  cancel-safety rules that make it correct.

- [Gabriel Gambetta: Client-Side Prediction and Server Reconciliation](https://gabrielgambetta.com/client-side-prediction-server-reconciliation.html)
  The standard explanation of showing a local change before the server confirms it, then
  reconciling. `PendingEdits` is a cut-down version of this. Use for: lesson on optimistic
  edits; also [Part III: Entity Interpolation](https://gabrielgambetta.com/entity-interpolation.html)
  for smoothing *other* people's movement, which is what presence (step 5) will need.

- [Gabriel Gambetta: Entity Interpolation (Part III)](https://www.gabrielgambetta.com/entity-interpolation.html)
  The canonical statement of the other smoothing strategy: buffer timestamped samples and
  draw at `now - delay`, so the client shows "actual movement data, except you're showing it
  100 ms late", using "only real entity data provided by the server". Its stated costs are
  the useful part for this project &mdash; three of the four (different world per player,
  shooting at moving targets, aiming lag) simply do not arise here. Lesson 0006 uses it to
  explain why `spring.ts` is the right choice instead, and why `CLAUDE.md`'s "drop outdated
  ones instead of queueing them" already forbids this technique.

- [Glenn Fiedler: What Every Programmer Needs To Know About Game Networking](https://gafferongames.com/post/what_every_programmer_needs_to_know_about_game_networking/)
  History and trade-offs of the authoritative-server model, from someone who shipped it.
  Use for: vocabulary and the reasoning behind server authority. Note the site is archived;
  the author has moved on, so treat dates with care.

- [Axum: `extract::ws::WebSocket`](https://docs.rs/axum/latest/axum/extract/ws/struct.WebSocket.html)
  "A stream of WebSocket messages." Documents `recv` as only "Receive another message.
  Returns `None` if the stream has closed." Notably says **nothing about cancel safety**,
  which is why lesson 0002 reads the source instead: `recv` is literally `self.next().await`
  and `WebSocket` implements `Stream`, so all framing state lives in `self.inner`
  (`axum-0.8.9/src/extract/ws.rs:555` and `:580`, in the local cargo registry). Use for: the
  worked example of deciding cancel safety for yourself.

- [Axum WebSocket example](https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs)
  The canonical shape of an axum socket handler, for comparing against `upgrade` and
  `client` in `crates/server/src/lib.rs`.

- [React: Render and Commit](https://react.dev/learn/render-and-commit)
  The maintainers' own account of the render cycle, in three named steps: trigger, render,
  commit. Two sentences carry lesson 0005: "Updating your component's state automatically
  queues a render" (nothing else does), and "React does not touch the DOM if the rendering
  result is the same as last time". The second is what makes putting `PendingEdits` in the
  store wasteful *by construction* rather than merely slow. Read before touching where state
  lives.

- [React: State as a Snapshot](https://react.dev/learn/state-as-a-snapshot)
  Why a render sees the state it was triggered with. The page most worth reading for someone
  arriving from a frame loop, where reading a mutable position at any time is normal.

- [Zustand: `createStore` (vanilla)](https://zustand.docs.pmnd.rs/apis/create-store)
  `getState` and `subscribe`, the two methods `KriegsspielEngine` uses to read the store
  without being a component. Lower trust than react.dev but it is the maintainers' own docs.

## Wisdom (Communities)

- [users.rust-lang.org](https://users.rust-lang.org/)
  Official Rust user forum. Genuinely high-signal on async and tokio questions; the
  cancel-safety threads found while writing lesson 1 came from here. Use for: "is this the
  right shape for my connection loop?"-type critique.

- [r/gamedev](https://reddit.com/r/gamedev) and [GameDev.net forums](https://www.gamedev.net/forums/)
  Where the client-side-prediction and reconciliation arguments play out in public. Use
  for: critique of the presence and interpolation design once it exists.

## Gaps

- No high-trust community found that is specifically about multiplayer networking
  architecture: the search turned up vendor Discords and general game-dev servers. Worth
  another look before step 5 (presence), where real critique would be most valuable.
- Nothing yet on relay/NAT traversal design (step 4), which is still an open question in
  the project.
- No primary source yet on WebSocket behaviour over long-haul links (proxy idle timeouts,
  keepalive), which the worldwide-play assumption in CLAUDE.md rests on. Lesson 0007 raised
  the stakes on this: structural cleanup cannot see a half-open connection, so heartbeats are
  now a named gap rather than a vague one. This is the strongest candidate for the next
  research pass.
