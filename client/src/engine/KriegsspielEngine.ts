import { DirectionalLight, Scene, WebGPURenderer } from "three/webgpu";
import type { Unit } from "../model";
import type { ServerConnection } from "../net/ServerConnection";
import type { ClientStore } from "../state/store";
import { CameraRig } from "./camera/CameraRig";
import type { EditorContext } from "./EditorContext";
import { KeyInput } from "./input/KeyInput";
import { Keymap } from "./input/Keymap";
import { Manipulation } from "./Manipulation";
import { OVERLAY_LAYER } from "./layers";
import { BLANK_TABLE, MapTable } from "./map/MapTable";
import { PawnLayer } from "./pawns/PawnLayer";
import { PendingEdits } from "./PendingEdits";
import { Picker } from "./picking/Picker";
import { MarqueeBox } from "./tools/MarqueeBox";
import { ToolInput } from "./tools/ToolInput";

/** Longest frame time the camera moves by, so a stalled or hidden tab does not cause a jump. */
const MAX_FRAME_SECONDS = 0.1;

export class KSEngine {
  private store: ClientStore;
  private renderer: WebGPURenderer;
  private rig: CameraRig;
  private scene: Scene;
  private table: MapTable;
  private pawns: PawnLayer;
  private picker: Picker;
  private keymap: Keymap;
  private tools: ToolInput;
  private keys: KeyInput;
  private marquee: MarqueeBox;

  /** Edits asked of the server and not yet seen in a board. */
  private pending = new PendingEdits();
  /** The units as drawn: the board with `pending` applied. Everything picks against these. */
  private shown: readonly Unit[] = [];
  /**
   * The selection being moved, turned or resized, while any drag or held key is still doing
   * so. At most one, so the inputs add up instead of each previewing the same units.
   */
  private manipulation: Manipulation | null = null;

  private resizeObserver: ResizeObserver;
  private unsubscribe: () => void;
  private disposed = false;
  /** Whether the camera has been put on the table since the first map arrived. */
  private framedTable = false;
  /** Whether the renderer is up, and so whether the table may fetch its images yet. */
  private ready = false;
  private lastFrameTime: number | null = null;

  constructor(
    canvas: HTMLCanvasElement,
    store: ClientStore,
    connection: ServerConnection,
  ) {
    this.store = store;
    this.renderer = new WebGPURenderer({
      canvas: canvas,
      antialias: true,
      // Selection outlines are masked with the stencil buffer.
      stencil: true,
    });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setAnimationLoop((time) => this.render(time));
    // The one place keys are turned into what they mean, for the tools and the camera alike.
    this.keymap = new Keymap(store.getState().bindings);
    this.rig = new CameraRig(canvas, store.getState().cameraMode, this.keymap);
    this.rig.camera.layers.enable(OVERLAY_LAYER);
    this.rig.frameArea(BLANK_TABLE);

    this.scene = new Scene();
    // Anisotropy is read when a texture loads, since the renderer can only answer for it
    // once it has initialised.
    this.table = new MapTable(
      () => this.renderer.getMaxAnisotropy(),
      // An image that will not load leaves a blank sheet, which looks exactly like a command
      // that never arrived, so it has to say so itself.
      (asset, error) =>
        this.store
          .getState()
          .setNotice(
            `Map image ${asset} could not be loaded: ${error instanceof Error ? error.message : error}`,
          ),
    );
    this.scene.add(this.table.group);

    const sun = new DirectionalLight(0xffffff, 3);
    sun.position.set(-40, 80, 60);
    this.scene.add(sun);

    this.pawns = new PawnLayer();
    this.scene.add(this.pawns.group);
    this.marquee = new MarqueeBox();
    this.scene.add(this.marquee.mesh);
    this.picker = new Picker(canvas, this.rig, () => this.shown);
    this.refresh();
    this.pawns.showSelection(store.getState().selection);

    const context: EditorContext = {
      pick: this.picker,
      units: () => this.shown,
      selection: () => store.getState().selection,
      readOnly: () => store.getState().viewing !== null,
      select: (ids, mode) => store.getState().select(ids, mode),
      manipulation: () => this.manipulation,
      manipulate: () =>
        (this.manipulation ??= this.startManipulation(connection)),
      showMarquee: (box) =>
        box
          ? this.marquee.show(box, canvas.getBoundingClientRect())
          : this.marquee.hide(),
      send: (message) => connection.send(message),
    };
    this.tools = new ToolInput(canvas, store, this.keymap, context);
    this.keys = new KeyInput(this.keymap, context);

    this.unsubscribe = store.subscribe((state, previous) => {
      if (state.cameraMode !== previous.cameraMode) {
        this.rig.setMode(state.cameraMode);
      }
      if (state.bindings !== previous.bindings) {
        // Nothing caches a binding, so the next event already uses these.
        this.keymap.bindings = state.bindings;
      }
      if (state.board !== previous.board) {
        // This board answers every pending edit no gesture is still holding.
        this.pending.settle();
        this.refresh();
        this.syncMaps();
      }
      if (
        state.board !== previous.board ||
        state.selection !== previous.selection
      ) {
        this.pawns.showSelection(state.selection);
      }
    });

    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(canvas);
    this.resize();
  }

  async init() {
    await this.renderer.init();
    if (this.disposed) return;
    this.ready = true;
    this.syncMaps();
  }

  dispose() {
    this.disposed = true;
    this.renderer.setAnimationLoop(null);
    this.unsubscribe();
    this.tools.dispose();
    this.keys.dispose();
    this.rig.dispose();
    this.table.dispose();
    this.pawns.dispose();
    this.marquee.dispose();
    this.resizeObserver.disconnect();
    this.scene.dispose();
    this.renderer.dispose();
  }

  private resize() {
    const canvas = this.renderer.domElement;
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    if (width === 0 || height === 0) return;
    this.renderer.setSize(width, height, false);
    this.rig.setSize(width, height);
  }

  /** One frame. `time` is the animation loop's timestamp in milliseconds. */
  render(time: number) {
    const dt =
      this.lastFrameTime === null
        ? 0
        : Math.min((time - this.lastFrameTime) / 1000, MAX_FRAME_SECONDS);
    this.lastFrameTime = time;
    this.rig.update(dt);
    this.keys.update(dt);
    // After the keys, so a held key's change this frame is what the blocks move towards.
    this.pawns.update(dt, this.manipulation?.lifted());
    this.renderer.render(this.scene, this.rig.camera);
  }

  /** A manipulation of the selected units as drawn, or null when nothing is selected. */
  private startManipulation(connection: ServerConnection): Manipulation | null {
    // Nothing is moved, turned or resized while a recorded turn is on screen.
    if (this.store.getState().viewing !== null) return null;
    const selected = new Set(this.store.getState().selection);
    const starts = this.shown.filter((unit) => selected.has(unit.id));
    if (starts.length === 0) return null;
    return new Manipulation(
      {
        preview: (edits) => {
          this.pending.hold(edits);
          this.refresh();
        },
        releasePreview: () => this.pending.release(),
        discardPreview: () => {
          this.pending.discard();
          this.refresh();
        },
        send: (units) => connection.send({ type: "place", units }),
        ended: () => {
          this.manipulation = null;
        },
      },
      starts,
    );
  }

  /**
   * Work out the units as drawn and draw them. Called when a board arrives and whenever a
   * pending edit changes, which during a drag is every move of the pointer.
   */
  private refresh() {
    this.shown = this.pending.applyTo(this.store.getState().board.pawns);
    this.pawns.sync(this.shown);
  }

  /**
   * Draw the maps the board has laid on the table. The camera is framed on them the first
   * time any arrive; later changes leave the camera where the umpire put it.
   *
   * Each map carries its own extent, so the table is the right shape, and can be framed,
   * without waiting for a single image to load.
   */
  private syncMaps() {
    if (!this.ready) return;
    const maps = this.store.getState().board.maps;
    this.table.sync(maps);
    if (maps.length > 0 && !this.framedTable) {
      this.framedTable = true;
      this.rig.frameArea(this.table.area());
    }
  }
}
