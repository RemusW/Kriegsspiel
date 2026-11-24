# Kriegsspiel Planner: project context for Claude

This file records what was decided and why, so a new session can continue without earlier
conversations.

**Keep this file current.** When a decision is made or reversed, an open question is settled,
or a milestone lands, edit the relevant section in place. Remove items that are no longer
true instead of appending corrections. Keep it short enough to read in full every session.

## What the project is

A tool for running Kriegsspiel games: an umpire edits a map and moves pieces on a 3D table;
players join over the network and see only their own side's view. The scene is 3D but
interaction is mostly 2D, flat on the map. Pieces are blocks with an image on the top face.

Secondary goal: this is also a portfolio piece. The owner wants it to demonstrate
TypeScript + React, three.js on WebGPU with hand-written shaders (TSL, with raw WGSL where it
helps), a Rust backend, real-time networking and Docker. When two approaches are otherwise
equal, prefer the one that exercises those skills. In particular, write the terrain shader
and geometry by hand; do not hide them behind a ready-made terrain library.

## Target user scenario

How a finished game should run. Design features against this.

1. **Hosting.** An umpire opens the desktop app and hosts a game. Other umpires join with
   umpire rights; players join from their browsers. Anyone can be anywhere in the world.
2. **Umpires edit together, live.** Umpires move pieces and see each other's edits and
   presence as they happen.
3. **Turns.** Umpires decide when a turn is ready and end it. Players then see what the
   umpires give them: their side's view as of that turn, not the umpires' work in progress.
4. **Team messages.** During a turn, players send messages to teammates. The server holds
   them, and they are processed when the turn ends.

What a worldwide spread means for live editing:

- Expect round trips of a few hundred milliseconds through the relay (an estimate). Edits
  must feel immediate for remote umpires, not only for the host:
  - A client shows its own edits at once. `PendingEdits` holds what this client has asked
    for and not yet seen in a board, and everything on screen is drawn, picked and measured
    through it. A refused command still sends its umpire a board (`Effect::Me`), so a pending
    edit can never stick. Request ids and a reply per command are still needed to say *why*
    something was refused or only partly applied (`Step::Partial`).
  - Whole-board snapshots stay. Per-unit deltas were considered and rejected: a delta is
    only correct for a client that has every earlier one, which the watch channel's
    coalescing deliberately does not guarantee, so they would mean a per-connection queue,
    sequence numbers and a resync path. Snapshots are self-healing instead. If the snapshot
    path needs to be cheaper, serialize once per distinct view rather than once per
    connection: umpires on the live board all receive identical bytes today, cloned and
    serialized once each.
  - What live editing needs is presence, not deltas. Edits are human-paced (one command per
    drag), so the 20 Hz stream of another umpire's drag belongs in the presence messages
    below, which are not part of `Doc` and never an `Edit`.
- Players only need updates when a turn ends, so a full filtered snapshot per turn is fine for
  them, and latency matters little.
- Dropped connections are normal over long distances. A reconnecting client resyncs from a
  fresh snapshot and keeps its identity, but not its undo history: undo covers what an
  umpire did while connected, and losing the connection loses it.

## About the owner

A graphics programmer by trade, most familiar with three.js and graphics concepts. Basic
knowledge of React, and little practice with networking, including the server side.

- Skip explanations of graphics and three.js basics.
- Treat React and networking work as a chance to teach. When either comes up, explain the
  concepts involved (what they are, why the code uses them, what goes wrong without them),
  tied to the code at hand. Where it helps, relate them to graphics ideas the owner already
  knows: React's render cycle against a frame loop, or smoothing network updates against
  interpolating between keyframes.

## Working with the owner

- Act as a senior software engineer. The main job is helping the owner make sound
  architectural decisions; writing code comes second.
- On a design question, recommend one approach and give its trade-offs: what it costs, what
  it rules out later, and which decisions or rules in this file it touches. Say so plainly
  when a request conflicts with this file or looks like a mistake.
- Before writing code beyond a small mechanical change, explain the approach (what changes,
  where, and why) and make sure the owner understands it, then implement.
- After implementing, walk through the key parts and point out anything worth a closer look.
  This is a portfolio piece, so the owner must be able to explain every part of it.

## Agent skills

### Issue tracker

Issues live as local markdown files under `.scratch/<feature>/` (gitignored for now). See `docs/agents/issue-tracker.md`.

### Triage labels

The five default roles (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`), recorded as a `Status:` line in each issue file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root, created only when needed. See `docs/agents/domain.md`.

## Repository layout

All TypeScript lives in `client/`, all Rust in `crates/`. The root holds only the two
workspace manifests (`Cargo.toml`, and a `package.json` with `client` as its one npm
workspace and scripts that call into it), their lockfiles and the docs.

| Path | What it is |
|---|---|
| `client/` | Web client (npm package `kriegsspiel-client`): React UI around a plain three.js engine. Runs in the Tauri window and in a plain browser. Builds to `client/dist` |
| `client/src/engine/` | `KriegsspielEngine`: owns the canvas, `WebGPURenderer`, camera and scene, and draws the units as shown rather than as stored. `App.tsx` mounts it once and hands it the store. `PendingEdits`: the edits asked of the server and not yet seen in a board. `Manipulation`: the selection being moved, turned and resized by any drags and held keys at once, sent as one `place` when the last input lets go; the engine keeps at most one. `EditorContext`: the one door the tools reach the engine through. `coords.ts`: screen and table geometry (`ScreenPoint`, `PressPoint`, `ScreenBox`, `TablePoint`) |
| `client/src/engine/actions/` | An `Action` is plain data (`history.ts`, `placement.ts`, `turns.ts`) listed in `ACTIONS`: a `PressAction` runs on each keydown, a `HoldAction` returns a `KeyHold` that runs every frame until its key comes up, from which `ActionId` and a keybinding menu's labels are derived. An action is what a key asks for, so it never names a key |
| `client/src/engine/camera/` | `CameraRig` owns the three.js camera and the controller for the current camera mode (`FlyController`, `MapController`); `KeyState` tracks held keys for both and answers whether a `Modifier` is held |
| `client/src/engine/pawns/` | `PawnLayer` keeps one block mesh per unit it is given, matched by id, each with a hidden selection outline as a child. It draws whatever units the engine hands it and knows nothing of drags beyond which units are held. Each block follows its unit on damped springs (`pawns/spring.ts`, fixed 240 Hz step): it slides, turns and resizes smoothly, is lifted and leans into its motion while a drag holds it (`Manipulation.lifted`), falls under gravity with a small bounce and squash when let go, and new units drop in. This is drawing only: picking and edits use the units' exact positions, which the blocks may trail for a moment. `hitTestUnits` finds the unit under a ray |
| `client/src/engine/input/` | `Modifier` is the vocabulary tools and the camera use for held keys (`extend`, `fast`); `Keymap` is the only thing that knows which keys those are, from a `Bindings` table kept in the store, which also maps `KeyBinding`s to actions; `KeyInput` is the window's one owner of keys that ask for something and runs the action a press is bound to |
| `client/src/engine/picking/` | `Picker` turns a point on the canvas into a camera ray, a point on the table or the unit under it, using the maths in `pawns/hitTest`. It picks against the units as drawn, so a unit mid-drag is picked where it is seen |
| `client/src/engine/materials/` | Hand-written TSL materials. `outline.ts`: the selection outline (see Decisions) |
| `client/src/engine/layers.ts` | `OVERLAY_LAYER`, the three.js layer for editor overlays such as outlines |
| `client/src/engine/tools/` | `ToolInput` turns left-button input on the canvas into clicks and drags (with pointer capture) for the tool chosen in the store. A `Tool` is plain data (`select.ts`, `marquee.ts`, `spawn.ts`) listed in `TOOLS`, from which `ToolId` and the toolbar's labels are derived, so a tool is defined in one place. A press and release is a `click`; a drag past the threshold asks the tool to `begin` a `Gesture` (`MoveDrag`, `MarqueeGesture`) from a `PressHit`, the unit and table point that were under the pointer when the button went down (the scene may have moved under the press point by the time it is a drag, for example a formation turning under a held key), which `ToolInput` drives and then drops, so no gesture state outlives its drag. Tools reach the engine only through `EditorContext`. `MarqueeBox` draws the marquee |
| `client/src/net/ServerConnection.ts` | The WebSocket to the server: reconnects, writes received boards into the store, and sends commands. One module-level instance, passed to the engine in `App.tsx`; `useGameConnection` opens and closes it. `sendBytes` sends an image as a binary message, the one thing a client sends that is not a command |
| `client/src/net/endpoint.ts`, `client/src/net/assets.ts` | Where the server is (`socketEndpoint`, `httpEndpoint`), and everything that travels as bytes: `assetUrl` for a stored image, `assetId` to hash a file into its id, `addMap`/`setUnitImage` to upload and then use one and `removeMaps` to take maps off, `saveGame`/`loadGame` for whole files |
| `client/src/engine/map/MapTable.ts` | The table top: one subdivided sheet per map on the board, kept by map id, plus a blank plane while there are none. `sync` is called with every board and only does what that board asks for: a sheet's geometry comes from the extent the board carries, so it is the right shape before its image has loaded, and the image itself is fetched by URL and owned by the sheet. `area` is what the camera is framed on |
| `client/src/state/store.ts` | Zustand store shared by the UI and the engine: active tool, selection, latest board, connection status |
| `client/src/interface/` | React UI: `Menu`, `GameScreen`, `Toolbar`, `Timeline` (the turn history bar, which sends its commands through `serverConnection` since it needs no engine), `FileBar` (adding maps and listing the ones on the table, unit images, save and open, and the one place a notice from the server is shown). One PascalCase file per component, styled by a CSS Module beside it (`Toolbar.module.css`). Shared colours, radii and fonts are CSS variables in `theme.css`, imported once in `main.tsx`. `App.tsx` only switches screens |
| `client/src/bindings/` | Board and message types generated by ts-rs: one file per Rust module (`model.ts`, `protocol.ts`, `assets.ts`), chosen by `#[ts(export, export_to = "<module>.ts")]` on each type. Never edit by hand: change the Rust types and run the full `cargo test -p kriegsspiel-server`, then commit the result. A filtered run that matches only some export tests (such as `cargo test board`) rewrites a file with just those types |
| `client/src/model.ts`, `client/src/protocol.ts` | Re-export the generated types, plus client-only constants (`EMPTY_BOARD`, `DEFAULT_PORT`) |
| `crates/server/` | `kriegsspiel-server`: the authoritative game server as a library, plus a standalone binary |
| `crates/server/src/assets.rs` | `AssetStore`: the images a game holds, keyed by the SHA-256 of their bytes. Reads each image's own header for its format and size (PNG and JPEG only) and never decodes pixels |
| `crates/server/src/save.rs` | `pack` and `unpack`: a save file's bytes in and out. Knows only `Doc` and `AssetStore`, so it could move to a crate of its own unchanged if a tool outside the server ever has to read a save |
| `crates/desktop/` | Umpire desktop app (Tauri; its docs call this folder `src-tauri`). Starts the game server in-process on port 7878 |

Client dependencies go in `client/package.json` (`npm install <pkg> -w client`); the root
`package.json` holds only the Tauri CLI. The Tauri CLI finds `crates/desktop/tauri.conf.json`
on its own and runs `beforeDevCommand`/`beforeBuildCommand` from the root.

Commands, from the repository root:

| Command | Does |
|---|---|
| `npm run tauri dev` | Desktop app with the embedded server and hot reload. In debug builds on macOS, Cmd+R (Develop > Reload) reloads the window |
| `npm run server` and `npm run dev` | Standalone server plus the client at `http://localhost:1420`, for working in a browser |
| `npm run build` | Typecheck and build the client |
| `cargo test -p kriegsspiel-server` | Server and model tests; also regenerates `client/src/bindings/` |

The client connects to `ws://<page host>:7878/ws`. The server sends a `snapshot` with the board
on connect and after every change; the client sends commands (`spawn`, `place`, `undo`, `redo`, `recordTurn`, `stepTurn`,
`viewTurn`, `restoreTurn`, `addMap`, `removeMaps` and `setUnitImage` so far). Both
server entry points listen on loopback only.

These HTTP routes carry CORS headers (any origin), because the client is never on the game
server's origin: `localhost:1420` in development, `tauri://localhost` packaged, the relay for
a player. Without them the browser withholds a response the server did answer — a map image
is fetched by three.js with `crossOrigin` and silently goes unused, and a `POST` never leaves
the browser because the preflight `OPTIONS` is refused. Allowing any origin is no weaker than
what already holds: CORS does not apply to the WebSocket, so any page can open one and send
commands today. That gap closes with umpire identity, not with CORS.

Alongside the socket, the same port serves what a socket is the wrong tool for:
`GET /assets/{id}` one image (immutable, so the browser caches it for good), `GET /save` the
whole game as one file, and `POST /load` to replace it. An image is uploaded as a binary
WebSocket message, answered only if refused (`assetRejected`); the command that uses it
follows as its own message, which needs no reply to wait for because a socket keeps order.

## Decisions

Why each choice was made is recorded in `docs/adr/`, one file per decision; the last column
names it. Record a new decision there and add its row here. Domain terms are defined in
`GLOSSARY.md`.

| Decision | Choice | ADR |
|---|---|---|
| World units | 1 world unit = 1 m on the ground. Each map carries its own ground scale (metres per pixel), set by calibrating against the map's scale bar, never from the image's DPI | 0001 |
| Client engine | three.js, over Babylon.js | 0002 |
| Scene layer | A plain three.js engine class that owns the canvas, mounted by one React component. Not React Three Fiber | 0003 |
| Renderer | `WebGPURenderer` from `three/webgpu`; custom materials written in TSL, with raw WGSL via `wgslFn` where needed | 0004 |
| UI | React | 0005 |
| Server | Rust, authoritative | 0006 |
| Desktop shell | Tauri | 0007 |
| Hosting | The umpire's desktop app runs the game server | 0008 |
| Player view | Real 3D in the browser, built from the baked terrain | 0009 |
| Transport | WebSockets first | 0010 |
| Storage | SQLite or plain files on the umpire's machine | 0011 |
| Relay | Small public service for lobby codes and connecting players to the umpire. Holds no game state; deployed with Docker | 0012 |
| Terrain delivery | Bake to 16-bit heightmap + colour texture; clients rebuild geometry | 0013 |
| Save format | One zip per game: `mimetype`, `manifest.json` (the whole document, plus an entry per image), and `assets/<id>.<ext>`. Extension `.ksg`, media type `application/vnd.kriegsspiel+zip`. Images stored uncompressed, the manifest deflated. All of it written and read in Rust; the client never unpacks a save | 0014 |
| Asset ids | The lowercase hex SHA-256 of an image's bytes | 0015 |
| Umpires | Several umpires per game. One umpire's app hosts the server; the others join over the network like players, with umpire rights | 0016 |
| Where edits run | On the server. Umpire clients send commands (move, rotate, undo, record turn and so on) and display the boards the server sends back; there is no client-side `Doc` | 0017 |
| Undo | Per umpire and per field, and only for as long as the connection lasts; see Data model | 0018 |
| Client state | A Zustand vanilla store. Components read it through `useClientStore`; the engine is handed the store and uses `getState` and `subscribe`. Per-frame data (camera, drag positions, presence) stays inside the engine, as does `PendingEdits`. Edits go to the server as commands and never change the stored board directly | 0019 |
| Shared types | ts-rs generates the client's board and message types from the Rust structs. `.cargo/config.toml` sets the output folder and exports `u64` as `number` | 0020 |
| Server updates | After a change, a `tokio::sync::watch` channel wakes every connection, and each sends its own view (`Doc::shown`, later filtered per side) as a whole-board `snapshot`. A snapshot carries the recorded turns (as `TurnInfo`: when each was recorded and how many units it holds, never their boards) and which of them this client is viewing, taken under one read of the document so the board and its label cannot disagree. Commands are applied by `apply` in `crates/server/src/lib.rs`, a plain function the tests call directly; it returns an `Effect` saying who must be sent a board: `Everyone` through the watch channel, `Me` straight back to the sender (a refusal, or a change only that umpire sees such as stepping turns), or `None` | 0021 |
| Selection outline | A yellow line 2 CSS px wide around the visible silhouette (`engine/materials/outline.ts`). The mesh is drawn again as a hull, its vertices pushed outward on screen along normals averaged over shared positions. Blocks write stencil 1 (`maskOutlines`); the hull draws after them, without depth test, only where stencil is not 1. The renderer is created with `stencil: true`. Anything that should hide an outline behind it, such as terrain and models, must call `maskOutlines` | 0022 |
| Camera controls | Two camera modes, chosen in the toolbar and kept in the store as `cameraMode`. **Fly** (default), like Tabletop Simulator's: W/A/S/D move horizontally along the camera's heading, R/F up and down, Shift faster, right-drag turns, the wheel moves along the view; speed grows with height. **Map**: three.js `MapControls` orbiting a table point with north up; right-drag or W/A/S/D pan, middle-drag tilts (never below 30 degrees), the wheel zooms to the cursor. Switching keeps the camera's place; map mode then turns it north-up | 0023 |

Size and timing figures in this file are estimates, not measurements. Replace them with
measured numbers once they exist.

## Open questions

- **How the relay connects players to the umpire.** Either the relay forwards WebSocket
  traffic both ways (simple, all traffic passes through it), or it only introduces the two
  sides and they connect directly over WebRTC (less relay load, more moving parts). Start with
  forwarding.
- **Visibility rules.** How the umpire decides what each side sees (manual per unit,
  per-side ownership, line of sight) is not designed yet.
- **Identifying umpires.** The server must stop a player from claiming umpire rights, and
  presence will need to say whose cursor is whose. It does not have to carry undo history
  across connections: that history is deliberately dropped with the connection, so identity
  is only about who may do what. Until it exists,
  nothing stops a page the umpire happens to be visiting from opening a WebSocket to their
  own server and sending commands, or from reading `/save`; the HTTP routes allow any origin
  for that reason, since being strict there would only move the hole, not close it. How an umpire
  proves who they are (a token from the host, an invite code) is not designed yet. Until
  then, every connection is treated as a new umpire. `Doc::forget_umpire` drops the entry
  when the connection ends, which is collection today and becomes the thing that enforces the
  rule once an id can come back. What identity does owe the client is a way to *see* that the
  history is gone: `Step::Empty` is `Effect::None`, so Ctrl+Z after a reconnect sends nothing
  at all and the key looks broken. A snapshot should say whether undo and redo are available,
  so the UI can grey them out.
- **Team messages.** What happens to messages when a turn ends is not designed: delivered as
  written, or reviewed by umpires first (delayed, lost or intercepted, as in classic
  Kriegsspiel). Also open: whether players send orders to the umpires the same way. Either
  way, messages are game state: saved with their turn and never sent to the other side.
- **Presence (planned).** Umpires should see each other work live: other umpires' cursors and
  selections, and pieces moving while another umpire drags them. Not designed yet. What it
  must respect:
  - Presence is transient. It is not part of `Doc`, not saved and not undoable. A drag still
    ends in one edit command; presence updates only show the drag while it happens.
  - Umpires only. A piece mid-drag shows where it is going, so the server never forwards
    presence to players, the same as any other hidden information.
  - Throttle updates (a starting guess is around 20 a second), let the receiving engine
    smooth between them, and drop outdated ones instead of queueing them.
  - Show which pieces another umpire is holding, so two umpires do not grab the same piece
    unawares. If they do, per-umpire undo already copes with the result.
- **Lua scripting for umpires (future goal).** Umpires write Lua to customise rules: actions
  run on a selection (resolve fire, roll morale), end-of-turn processing, and later
  visibility rules. Not needed soon, and a first version can be small, such as actions only.
  It comes after step 4 at the earliest, since it needs edit commands, umpire identity and
  the filtering step. Planned runtime: Luau through the `mlua` crate, with its sandbox, a
  memory cap and a time limit. Luau is C++, so reconsider (Rhai and Starlark are pure Rust)
  if the server ever has to compile to WebAssembly. To keep it cheap to add:
  - Scripts run only on the server, never in player clients. Only umpires can upload or
    trigger them.
  - A script run is one edit: run it on a copy of the live board, then pass the result to
    `Doc::edit`. A failing script changes nothing, and undo needs no script support. Keep
    `Doc::edit` accepting any board change; do not narrow it to a fixed set of edit types
    when commands are added.
  - Scripts change units only through an API. `spawn` assigns ids; scripts never set `id`
    or `nextId`. Check the board after each run (unique ids, finite numbers).
  - Rule data such as strength or morale goes in a per-unit property map (`props`: names to
    numbers, text or booleans), and undo merges it per key. This changes `Unit` on both sides
    and the save format, so add it before save files have to stay loadable.
  - Keep per-side filtering in one server function, so a visibility script can replace it.
    Filtering may need to hide properties, not only units.
  - Script source is stored in the save. Dice rolls come from the server and are logged for
    umpires.
  - Scripts declare their actions and properties as data, and the React client renders
    them. Scripts never draw UI.
  - A script holds the document lock while it runs, so its time limit must be short.
- **Agent control through MCP (future goal).** An AI agent such as Claude Code controls the
  board as one more umpire client: a `crates/mcp` crate using `rmcp` over stdio connects to
  the game server's WebSocket with umpire rights and turns each tool call into an ordinary
  command, plus read tools such as `get_board`. It goes through the same checks, undo and
  filtering as a person; with player rights it would be a fair AI opponent. Not inside the
  game server, so there is no second way in. It comes after step 4. To keep it cheap to add:
  - Commands describe intent, not input ("move units 3 and 5 to these positions"). Every
    board change is a command, and a multi-unit move is one command.
  - Every command has a request id and gets a reply: applied, partial or refused, with the
    reason.
  - The server validates every command: unknown ids, non-finite numbers, off-map positions.
  - Identity covers non-human clients with named, revocable tokens. An agent gets its own
    umpire identity, and the umpire who invited it can undo its steps.
  - Commands derive `schemars::JsonSchema` next to `TS`, so the MCP tool schemas come from
    the same Rust types. Doc comments on command fields become tool descriptions.

Consequences of the hosting decision to keep in mind:

- **Keep the game server a library crate.** The Tauri app embeds it; the standalone binary
  wrapping the same crate keeps a dedicated host possible.
- **Players reach the umpire three ways:** same network (direct), port forwarding (direct),
  or through the relay (everyone else). The player page must be served so that secure-page
  rules do not block the connection; serving it from the relay over HTTPS with a secure
  WebSocket to the relay is the straightforward route.
- **A game pauses when the umpire's app closes.** State lives on the umpire's machine, so
  save on every turn and let players reconnect to a resumed game.
- **Possible future goal: a cloud-hosted server** (deferred). It would serve play-by-post
  games that run for weeks, where players and umpires check in while the host is offline.
  The same server crate would run as a persistent service on one small VPS in Docker, with
  saves on disk and offsite backups. Everyone would connect to it directly, so the relay
  would no longer be needed. It needs a lasting identity per player and umpire, such as a
  private link containing a secret. Estimated at roughly $6–20 a month (Nov 2026 prices). To
  keep this cheap to add later:
  - Keep the server crate free of Tauri dependencies.
  - Pass the storage location in from outside.
  - Never assume the host umpire is on the same machine as the server.
- **Tauri renders through the system web view.** Linux's WebKitGTK probably has no WebGPU,
  so the desktop app there would run on the WebGL2 fallback. Test the 3D scene on Linux early,
  and test the fallback elsewhere with `forceWebGL: true`. Players use their own browsers, so
  this does not limit them.

## Architecture rules to keep

- **The board is a plain serializable document; the renderer only displays it.** Never make
  scene objects the source of truth. Turn history, save files and network messages are
  snapshots of this document; undo steps record per-unit before and after values.
- **Every edit goes through `Doc::edit` on the server,** so it becomes one undo step for the
  umpire who made it. Mutating `doc.board` directly is only for setup, such as the demo board.
- **Unit ids are never reused** within a game, not even after undo or restoring a turn. Undo
  steps refer to units by id.
- **Filter on the server.** A player's client must never receive data for units it may not
  see. Do not send the full board and hide things client-side.
- **Saves are self-contained.** Images live in the save itself, referenced by the SHA-256 of
  their bytes, never by local file path. A command may only name an image already in the
  store, so a board can never refer to one a save would not find; `pack` carries every image
  the live board or any recorded turn names and leaves out the ones nothing uses. The server
  writes and reads save files, because it is the only thing holding both the document and the
  images: the client asks for the bytes or hands them over, and waits for the board that
  comes back.
- **A drag is one undo step.** The engine moves the dragged meshes locally and the client
  sends one command when the drag ends, and only if it passed the movement threshold. Never
  send a command per frame or on mouse-down. Presence updates during a drag (see Open
  questions) are separate messages, not edits. Held keys that change units (turn, resize)
  join the same `Manipulation` as the drag, so dragging while turning, or letting go of the
  mouse and carrying on turning, is still one command and one undo step, sent when the last
  input lets go. Losing the pointer mid-drag cancels the whole manipulation.
- **Two keyboard readers, each with one job.** `KeyInput` acts on bound keys: it runs a press
  action, or drives a hold action from its keydown to its keyup; `KeyState` only tracks which keys are *held*, for the camera. A binding therefore
  cannot be claimed twice, and `KeyState` can keep dropping anything pressed with Ctrl, Cmd or
  Alt so Ctrl+Z never also flies the camera.
- **Input is named by what it means, never by which key it is.** A tool asks whether the
  `extend` modifier is held, not whether Shift is down, and `Keymap` turns the keys an event
  carried into `Modifier`s from a `Bindings` table. Keys appear only in `engine/input/`, so
  rebinding is a change to data and a settings menu is one `setBindings` call. A `PressPoint`
  carries the modifiers resolved at its own event, because a key held at the press may be
  released before the drag ends; which of the two a gesture uses is a choice to state in its
  doc comment, not an accident of the point that got passed.
- **What is on screen is the board plus `PendingEdits`.** Drawing, picking and the values a
  new edit builds on all read the units as drawn, never `board.pawns` directly, which lags
  the screen by a round trip. Otherwise a second drag inside one round trip measures from a
  stale position and undoes the first, and a click lands where a dragged piece used to be. A
  gesture holds its own pending edits, so a board sent for another umpire's change cannot
  clear a drag that is still under way; every other pending edit is dropped when a board
  arrives, and a refusal sends one too.
- **Screenshots exclude the editor overlay.** Hide UI and highlights before capturing.
  Overlay meshes go on `OVERLAY_LAYER`, so a capture only has to disable that camera layer.
- **Node materials only.** Import three.js classes from `three/webgpu` and shader functions
  from `three/tsl`, never from `three`. `ShaderMaterial`, `RawShaderMaterial` and
  `onBeforeCompile` do not work with `WebGPURenderer`. Built-in materials such as
  `MeshBasicMaterial` are converted to node materials automatically.

## Code comments

- Keep comments simple and concise. Use them to explain what the code cannot say by itself:
  why something is done, an invariant, a non-obvious consequence.
- Do not restate what self-documenting code already says through its names and types.
- Leave requirements, decisions and plans ("for now", "comes with the lobby work") out of
  comments. They belong in this file.

## Data model

```ts
type UnitKind = "Infantry" | "Cavalry" | "Artillery";
type Side = "Blue" | "Red";

// Placement shared by every piece on the table.
interface Pawn {
  pos: [number, number]; // map plane: world X, world Z
  rot: number;           // radians around the vertical axis
  scale: number;
  image: string | null;  // asset id; null uses the generated symbol
}

// In Rust the Pawn is a `pawn` field with `#[serde(flatten)]`, so the JSON stays flat.
interface Unit extends Pawn {
  id: number;            // stable across snapshots; selection is stored as ids
  kind: UnitKind;
  side: Side;
}

// One image laid on the table. Any number may be down at once.
interface MapImage {
  id: number;            // from the same counter as the units'
  asset: string;         // asset id; a map is its image, so never null
  pos: [number, number]; // centre of the sheet: world X, world Z
  size: [number, number]; // extent of the sheet; always the image's aspect ratio
}

interface Board {
  maps: MapImage[];      // bottom to top: a later sheet is drawn over an earlier one
  pawns: Unit[];
  nextId: number;        // ids for everything on the board, maps and units alike
}

// Server only (Rust), not sent to clients as a whole.
interface Doc {
  board: Board;          // the live board
  // Snapshots recorded when an umpire chooses, each stamped with the wall-clock time the
  // server received the command (`recordedAt`, milliseconds since the epoch).
  turns: { board: Board; recordedAt: number }[];
  // Per umpire: an undo and a redo stack of edits, and the recorded turn they are viewing.
  umpires: Map<UmpireId, { undo: Edit[]; redo: Edit[]; viewing: number | null }>;
}
```

Behaviour to preserve:

- **Undo/redo:** per umpire, each stack capped at 200. Each step records the touched units
  before and after. An umpire's new edit clears only their own redo. Undo restores each field
  only where the board still holds the value that edit left. Fields another umpire changed
  since are skipped and the result says so (`Step::Partial`). Undoing a spawn keeps the unit
  if someone else has changed it since. Redoing a partial undo repeats only the part that was
  undone. **History lasts as long as the connection.** When one ends, the document drops
  what it kept for that umpire, history and viewed turn alike (`Doc::forget_umpire`), and a
  reconnecting umpire starts empty even once they can prove who they are. Undo is not game
  state: it is also absent from a save and cleared by a load. Nothing a client is sent names
  an umpire, so unlike every other write to the document this one wakes no connection.
- **Turns:** recording appends a copy of the live board, stamped with the time. Each umpire
  steps through turns on their own, or jumps to one by index (`viewTurn`, with `null` for the
  live board): stepping back from live goes to the last turn; stepping forward past the last
  turn returns to live. Viewing a turn is read-only for that umpire: their edits, undo and
  recording are refused, and a refusal still answers them, because their timeline must show
  what they are really looking at. The client offers nothing it knows would be refused:
  `EditorContext.readOnly` turns off spawning, manipulation and history while a turn is on
  screen, but not selection, since the past is there to be looked at. "Restore" copies the
  viewed turn into the live board as one undoable edit and returns that umpire to live.
- **Maps:** `addMap` lays a stored image on the table as a new map and `removeMaps` takes
  maps off; both are ordinary undoable edits, merged per field like units, and undoing a
  removal puts the sheet back at its old height in the stack. Only the server sets a map's
  `size`, from the pixel size in the image's own header, so a sheet always keeps its image's
  aspect ratio and every client agrees on where its edges are without having loaded it. A new
  map is `NEW_MAP_WIDTH` (100) wide and goes east of every map already down, clear of them,
  because nothing moves maps yet.
- **Loading a file** replaces the document and clears every umpire's history. A save written
  before boards held more than one map (format 1) is still read: `save.rs` lays its single
  `mapImage` down, sized from the image in the file.
- **Selection** is client-side, per user, and pruned to ids that exist in the shown board after
  any change. It changes only through the store's `select(ids, mode)` (replace, add or
  toggle), which ignores ids that are not on the board.

`Doc` lives in `crates/server/src/model.rs`, with tests. The client has only the generated
`Board`, `MapImage`, `Unit`, `Pawn`, `UnitKind` and `Side`. Edit commands are `ClientMessage`
variants in
`crates/server/src/protocol.rs`; `spawn`, `place`, `undo` and `redo` exist so far. `place`
carries absolute values, one `Placement` per unit with any of `pos`, `rot` and `scale` (left-out
fields stay as they are), so the command states the result it wants and a repeat is harmless.
Drags, turns and resizes all send it. The server refuses non-finite values and scales outside
`MIN_SCALE`..`MAX_SCALE` (0.25 to 4, mirrored in `client/src/model.ts`).

## Conventions

- Y is up. The maps lie on the XZ plane at y = 0. North (top of a map image) is -Z.
- A map image is laid down 100 world units wide, its depth following the image's aspect
  ratio, and the blank default table is 100 x 70, shown whenever `board.maps` is empty. Any
  number of maps may be down at once; each carries its own centre and extent, and sheets are
  drawn in board order, each lifted a hair (0.005) above the one below so coplanar sheets do
  not fight for depth. The camera is framed on everything the maps cover the first time any
  arrive; later changes leave it where the umpire put it.
- An unscaled unit block is 4 wide, 0.5 thick, 2 deep. `scale` multiplies width and depth;
  thickness stays the same.
- Pieces may be placed and moved off the map, for when players need more room. The table is
  the plane y = 0; off the map, pieces rest on it.
- Hit testing a unit: intersect the cursor ray with the horizontal plane at the unit's top
  face, rotate the hit into the unit's local frame, compare against its half-extents. Where
  blocks overlap, the one whose centre is nearest the hit wins. Side faces are not tested.
- Box select picks units whose top-face centre projects inside the screen rectangle, on
  release. The marquee is a quad that a TSL vertex shader places straight in clip space
  (`MarqueeBox`), on `OVERLAY_LAYER`; the box being dragged never goes through the store.
- Turning (`Q`/`E`) wheels the selection about the centroid of its positions, which travels
  with a drag in progress: each unit moves around it and turns by the same angle, so a
  formation keeps its shape, and a single unit turns in place. A press turns 5 degrees; after 0.25 s held it turns 90 degrees a second.
  Positive `rot` is counterclockwise seen from above, as three.js's rotation about +Y.
  Resizing (`=`/`-`) scales each unit's footprint in place, by 1.05 a press and 1.5 times a
  second held, clamped to the allowed scales. Each input (drag offset, angle, log of the scale
  factor) only adds to a total, and each frame recomputes from the units as they were when the
  manipulation began, so rounding cannot build up. While a manipulation runs, its units are
  fixed: a drag that grabs a unit outside it does nothing.
- Default unit textures are generated: a side-coloured counter with a border and a symbol
  (crossed diagonals for infantry, one diagonal for cavalry, a dot for artillery).

## Planned editor controls

Camera controls are under Decisions. The keys below are the default bindings, and the ones
that stand for a meaning rather than a command (Shift, for `extend` and `fast`) are already
rebindable through `Bindings`. For editing: `1`/`2`/`3` spawn at cursor; `Tab`
switches side; click selects, Shift-click toggles a unit, drag moves, Shift-drag a unit adds
it to the selection and moves the lot; the marquee tool draws a box or lasso (chosen in the
toolbar) and selects what it encloses, Shift adds. Done so far: click, Shift-click, the box
marquee and drag to move. The lasso is not built: lasso mode draws a box. `Q`/`E` turn the
selection counterclockwise/clockwise and `=`/`-` grow/shrink it, both done (see Conventions);
`Delete` removes; Ctrl+Z undoes and Ctrl+Shift+Z redoes, both done, each umpire over their own
history; Cmd/Ctrl+A select all; `,`/`.` step through turns, done. Recording a turn and
restoring the viewed one are buttons on the timeline bar and are bound to no key: both are
bindable actions, so a settings menu can give them one. `P`
screenshots, Shift+`P` screenshots a dragged region.

File import and saving are done, in `FileBar`: buttons to add a map and to put an image on
the selected units, Save and Open, plus dropping a file anywhere on the window (an image is
laid on the table, anything else is tried as a save). The maps on the table are listed with
their extents, each with a button to take it off; nothing moves or resizes a map yet, so they
sit where they were laid. None of it is bound to a key yet, so
Cmd/Ctrl+S and Cmd/Ctrl+O are still free to collide with browser defaults when they are;
check that and Tab when binding them. The desktop app uses the same download and upload path
as the browser; a native Save As dialog through `tauri-plugin-dialog` is not wired up.

## Suggested order of work

1. Done: project scaffold (client, server library crate, Tauri shell embedding it, shared
   message types).
2. Done: document model with tests.
3. Editor basics: camera, map plane, units, select/move, undo, turns, screenshot, save/open
   with embedded images. Done except the screenshot.
4. Server and lobby: authoritative board, per-side filtered views, players joining on the
   same network first; then the relay and join-by-code over the internet.
5. Presence for umpires: live cursors, selections and drags.
6. Team messages, held until the turn ends.
7. Terrain: heightmap geometry, custom TSL shader, map baking, placed models.
8. Installers for Windows, Linux and Mac; Docker deployment of the relay.

## Progress log

Update this as milestones land. One line each, newest first.

- 2026-11-23: Undo history lasts exactly as long as the connection, and a finished
  connection no longer leaks the umpire it was. `client` now wraps `serve_client`, so the
  loop's five exits meet in one place, and `Doc::forget_umpire` drops that umpire's stacks and
  viewed turn there. Decided deliberately rather than deferred: undo is already absent from
  saves and cleared by a load, so keeping it across a reconnect would have made it more
  durable than the game it belongs to. Identity therefore owes the client only a way to see
  that undo is empty, not a grace period.

- 2026-11-20: Any number of maps on the table. `Board::map_image` became `Board::maps`, a
  list of `MapImage` (id, asset, centre, extent) sharing the units' id counter; `setMapImage`
  became `addMap` and `removeMaps`, both undoable and merged per field, with a map's place in
  the stack kept when a removal is undone. The server sets a sheet's extent from the pixel
  size in the image's own header, so aspect ratio is right by construction and the client
  draws the geometry before the image loads; `MapPlane` became `MapTable`, which keeps a
  sheet per map id and syncs against every board. Save files are format 2, and a format 1
  save still loads with its one `mapImage` laid down. Checked over the wire: two images of
  different shapes laid side by side, saved, loaded back, one taken off and undone, and an
  id nobody uploaded refused.

- 2026-11-20: The HTTP routes answer cross-origin requests (`tower-http`'s `CorsLayer`, any
  origin). Without it a map image loaded with `crossOrigin` was discarded by the browser and
  the table stayed blank although the upload had worked, and `POST /load` never left the
  browser at all because the preflight was answered 405. A failed map image now shows a
  notice instead of leaving a blank table, dropped file handling needs
  `"dragDropEnabled": false` to reach the webview at all, and the WebSocket's frame limit is
  raised alongside its message limit (the 16 MiB default would kill the connection rather
  than refuse an image).
- 2026-11-20: Self-contained save files. `AssetStore` holds a game's images by content hash
  and serves them from `/assets/{id}`; `save.rs` packs the document and the images it uses
  into one zip (`manifest.json` plus `assets/<id>.<ext>`) and reads it back, refusing a newer
  format, a manifest that disagrees with its own images, and a board naming an image the file
  does not hold. `setMapImage` and `setUnitImage` are ordinary undoable commands that may
  only name an image already uploaded; an upload is a binary WebSocket message and is not an
  edit, so it wakes nobody. `GET /save` and `POST /load` move whole files; `FileBar` has the
  buttons and takes dropped files. The client no longer bundles `farley.png`, so an empty
  board starts on the blank table. Checked over the wire against a running app: upload, lay
  the map, serve it, save (1.1 KB of overhead on a 15.7 MB file), load it back, and refuse
  both a non-image and a file that is not a save.

- 2026-11-19: Pawns move on springs: lift and lean while dragged, drop with a bounce and
  squash, slide to remote changes, and new units drop onto the table. Drawing only.
- 2026-11-19: Turn history. `recordTurn`, `stepTurn`, `viewTurn` and `restoreTurn` commands;
  `Doc::turns` holds a `Turn` (board plus the time it was recorded) and gained `view_turn`;
  every snapshot carries the turn list and the turn this umpire is viewing, so one message
  describes the whole view. A `Timeline` bar beside the toolbar steps through turns, lists
  them in a dropdown, records one, and offers Restore and Go live from the past; `,` and `.`
  step from the keyboard. Checked over the wire that stepping is per umpire and that edits,
  undo and recording from the past are refused.
- 2026-11-19: Turn and resize the selection from the keyboard. `move` became `place`, whose
  `Placement`s carry optional `pos`, `rot` and `scale`. `Q`/`E` and `=`/`-` are hold actions:
  one step on the press, continuous after a short pause, one `place` command on release.
  Turning wheels the selection about its centroid like a formation; resizing is per unit.
  A drag and held keys add up in one `Manipulation`, so the selection can be dragged and turned
  at once and still be one `place` command.
- 2026-11-19: Undo and redo are reachable: `Undo`/`Redo` commands on the server (an empty
  history is `Effect::None`, a read-only one `Effect::Me`), and on the client an `ACTIONS` list
  with `KeyInput` running the action a key is bound to. Ctrl+Z and Ctrl+Shift+Z by default,
  both rebindable through `Bindings`. Verified over the wire that each umpire takes back only
  their own edits.
- 2026-11-19: Modifier keys are named by meaning. `Modifier`/`Keymap`/`Bindings` replace the
  `shift` boolean on a press, the camera's fast movement asks for `fast`, and the bindings sit
  in the store ready for a settings menu. No key is named outside `engine/input/`.
- 2026-11-19: Tools are plain data in one `TOOLS` list, a drag is a `Gesture` object that
  cannot outlive its drag, picking moved into `Picker`, and `PendingEdits` makes what is on
  screen (board + edits in flight) the thing that is drawn, picked and measured from. `apply`
  now returns an `Effect`, so a refused command answers its sender instead of going silent.
- 2026-11-18: Drag to move: dragging a unit moves the selection (or that unit alone) with the
  select and marquee tools, and Shift-dragging an unselected unit adds it to the selection and
  moves the lot in the same gesture. One `move` command per drag carries every unit's new
  position, so it is one undo step; the blocks follow the pointer locally until the server's
  board arrives.
- 2026-11-17: Marquee tool: dragging a box selects the units inside, Shift adds; Shift-click
  toggles a unit. The box is drawn by a TSL shader.
- 2026-11-17: Select tool (single select): a click selects the unit under the cursor or
  clears the selection. Selected units get a yellow outline from a TSL shader.
- 2026-11-17: Spawn tool: a click on the map sends a `spawn` command; the server applies it
  as an undoable edit and sends every connection its new view. The engine draws units as
  side-coloured blocks.
- 2026-11-16: The engine draws the Farley map on a subdivided plane with one directional
  light. The camera has fly and map modes, switched from the toolbar.
- 2026-11-16: Board's `units` renamed to `pawns`, with placement fields split into a flattened
  `Pawn`. Client board and message types are now generated by ts-rs.
- 2026-11-16: Main menu shown before the game screen, a left toolbar (select; marquee with
  box or lasso) and a Zustand store. The toolbar reads and sets the tool through the store;
  the engine does not use it yet.
- 2026-11-15: `Doc` keeps undo, redo and turn viewing per umpire, with field-level undo that
  skips other umpires' later changes.
- 2026-11-15: Repository split into `client/` (TypeScript) and `crates/` (Rust).
- 2026-11-14: Project scaffold: the client shows the server's demo board, in a browser and in
  the Tauri app.
