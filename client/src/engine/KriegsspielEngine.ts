import {
  DirectionalLight,
  Mesh,
  MeshStandardMaterial,
  PlaneGeometry,
  Scene,
  SRGBColorSpace,
  TextureLoader,
  WebGPURenderer,
} from "three/webgpu";
import farleyUrl from "../../assets/farley.png";
import type { Unit } from "../model";
import type { ServerConnection } from "../net/ServerConnection";
import type { ClientStore } from "../state/store";
import { CameraRig } from "./camera/CameraRig";
import type { EditorContext } from "./EditorContext";
import { KeyInput } from "./input/KeyInput";
import { Keymap } from "./input/Keymap";
import { Manipulation } from "./Manipulation";
import { OVERLAY_LAYER } from "./layers";
import { PawnLayer } from "./pawns/PawnLayer";
import { PendingEdits } from "./PendingEdits";
import { Picker } from "./picking/Picker";
import { MarqueeBox } from "./tools/MarqueeBox";
import { ToolInput } from "./tools/ToolInput";

/** Width of the map plane in world units. Its depth follows the image's aspect ratio. */
const MAP_WIDTH = 100;
/** Depth of the blank default map, framed until a map image has loaded. */
const BLANK_MAP_DEPTH = 70;
/** No units held up off the table. */
const NOTHING_LIFTED: ReadonlySet<number> = new Set();
/** Longest frame time the camera moves by, so a stalled or hidden tab does not cause a jump. */
const MAX_FRAME_SECONDS = 0.1;

export class KSEngine {
  private store: ClientStore;
  private renderer: WebGPURenderer;
  private rig: CameraRig;
  private scene: Scene;
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
    this.rig.frameMap(MAP_WIDTH, BLANK_MAP_DEPTH);

    this.scene = new Scene();
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
    await this.exampleScene();
  }

  dispose() {
    this.disposed = true;
    this.renderer.setAnimationLoop(null);
    this.unsubscribe();
    this.tools.dispose();
    this.keys.dispose();
    this.rig.dispose();
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
    this.pawns.update(dt, this.manipulation?.lifted() ?? NOTHING_LIFTED);
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

  /** The Farley map on a subdivided plane, lit by one directional light and seen from above at an angle. */
  async exampleScene() {
    const texture = await new TextureLoader().loadAsync(farleyUrl);
    // The engine may have been disposed while the image loaded.
    if (this.disposed) {
      texture.dispose();
      return;
    }
    texture.colorSpace = SRGBColorSpace;
    texture.anisotropy = this.renderer.getMaxAnisotropy();

    const depth = MAP_WIDTH * (texture.image.height / texture.image.width);

    // One quad per world unit. Rotated onto the XZ plane facing up, which puts the top of
    // the image (north) towards -Z.
    const geometry = new PlaneGeometry(
      MAP_WIDTH,
      depth,
      MAP_WIDTH,
      Math.round(depth),
    );
    geometry.rotateX(-Math.PI / 2);
    const material = new MeshStandardMaterial({ map: texture, roughness: 1 });
    this.scene.add(new Mesh(geometry, material));

    const sun = new DirectionalLight(0xffffff, 3);
    sun.position.set(-40, 80, 60);
    this.scene.add(sun);

    this.rig.frameMap(MAP_WIDTH, depth);
  }
}
