# Kriegsspiel

A tabletop-simulator-style tool for running Kriegsspiel games. An umpire edits a map and moves
pieces on a 3D table; players join from a browser and see only what their side is allowed to
see.

The scene is 3D, but most interaction is 2D, flat on the map. Pieces are simple blocks with an
image on the top face.

## Requirements

- Node.js 22 or newer
- Rust (stable)
- The [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform

## Running

```sh
npm install

# Desktop app, with the game server inside it
npm run tauri dev

# Or, to work in a browser: the server and the client separately
npm run server
npm run dev        # then open http://localhost:1420
```

## Checks

```sh
npm run build                      # typecheck and build the client
cargo test -p kriegsspiel-server   # server and model tests
```

## Roadmap

- [ ] Editor: spawn, select, move, rotate and resize units; undo/redo; box and lasso select
- [ ] Turn recording and stepping through past turns
- [ ] Self-contained save files with embedded map and unit images
- [ ] Screenshots of the full view or a selected area
- [ ] Network play with per-side views, joined by lobby code
- [ ] Chat tied to turns
- [ ] Terrain from baked heightmaps, with placed models
- [ ] Installers for Windows, Linux and Mac
