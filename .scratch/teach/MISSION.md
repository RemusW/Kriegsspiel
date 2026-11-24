# Mission: The Kriegsspiel client-server network model

## Why

This project is a portfolio piece whose stated selling points include real-time
networking, and the owner is a graphics programmer who has little practice on the server
side. Much of the networking code was written with an agent's help, which is a liability at
an interview and a worse one when building on top of it. The goal is to own it: read every
line unaided, know which parts are general networking craft and which are this project's
choices, and defend both out loud.

## Success looks like

- Whiteboarding the whole data path from an umpire's drag to another umpire's screen, from
  memory, naming each hop.
- Answering "why a WebSocket and not HTTP?", "why does each connection build its own
  snapshot instead of broadcasting one?", and "what happens when the connection drops?"
  without hedging.
- Designing presence (step 5) and the relay (step 4) without needing an agent to make the
  architectural calls.
- Recognising the general pattern behind each project-specific choice: optimistic update,
  reconciliation, authoritative server, fan-out, idempotent commands.

## Constraints

- Graphics and three.js basics are already known; relate new network ideas to graphics ones
  (a frame loop, keyframe interpolation, double buffering) where it genuinely helps.
- Lessons are grounded in this repository's real code, never in toy examples.
- Little prior networking or server-side practice. Do not assume familiarity with async
  Rust, TCP semantics or HTTP caching.

## Out of scope

- Lua scripting, MCP agent control, terrain baking: all deferred in the project itself.
- Deep async-Rust theory (pinning, executor internals) beyond what reading the server needs.
- Per-side visibility rules, which are an open design question in the project, not a
  teachable fact yet.
