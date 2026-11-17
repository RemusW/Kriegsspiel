import {
  DirectionalLight,
  Mesh,
  MeshStandardMaterial,
  Plane,
  PlaneGeometry,
  Raycaster,
  type Ray,
  Scene,
  SRGBColorSpace,
  TextureLoader,
  Vector2,
  Vector3,
  WebGPURenderer,
} from "three/webgpu";
import farleyUrl from "../../assets/farley.png";
import type { ServerConnection } from "../net/ServerConnection";
import type { ClientStore } from "../state/store";
import { CameraRig } from "./camera/CameraRig";
import { OVERLAY_LAYER } from "./layers";
import { hitTestUnits } from "./pawns/hitTest";
import { PawnLayer } from "./pawns/PawnLayer";
import { ToolInput } from "./tools/ToolInput";

/** Width of the map plane in world units. Its depth follows the image's aspect ratio. */
const MAP_WIDTH = 100;
/** Depth of the blank default map, framed until a map image has loaded. */
const BLANK_MAP_DEPTH = 70;
/** Longest frame time the camera moves by, so a stalled or hidden tab does not cause a jump. */
const MAX_FRAME_SECONDS = 0.1;

const GROUND = new Plane(new Vector3(0, 1, 0), 0);

export class KSEngine {
  private store: ClientStore;
  private renderer: WebGPURenderer;
  private rig: CameraRig;
  private scene: Scene;
  private pawns: PawnLayer;
  private tools: ToolInput;

  private raycaster = new Raycaster();
  private pointer = new Vector2();
  private hit = new Vector3();

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
    this.rig = new CameraRig(canvas, store.getState().cameraMode);
    this.rig.camera.layers.enable(OVERLAY_LAYER);
    this.rig.frameMap(MAP_WIDTH, BLANK_MAP_DEPTH);

    this.scene = new Scene();
    this.pawns = new PawnLayer();
    this.pawns.sync(store.getState().board.pawns);
    this.pawns.showSelection(store.getState().selection);
    this.scene.add(this.pawns.group);

    this.tools = new ToolInput(canvas, store, {
      pickTable: (event) => this.pickTable(event),
      pickUnit: (event) => this.pickUnit(event),
      select: (ids) => store.getState().setSelection(ids),
      send: (message) => connection.send(message),
    });

    this.unsubscribe = store.subscribe((state, previous) => {
      if (state.cameraMode !== previous.cameraMode) {
        this.rig.setMode(state.cameraMode);
      }
      if (state.board !== previous.board) {
        this.pawns.sync(state.board.pawns);
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
    this.rig.dispose();
    this.pawns.dispose();
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
    this.renderer.render(this.scene, this.rig.camera);
  }

  /**
   * The point on the table under the pointer as [world X, world Z], on or off the map. Null
   * when the pointer is above the horizon or the point is beyond the camera's far plane.
   */
  private pickTable(event: PointerEvent): [number, number] | null {
    const ray = this.rayThrough(event);
    // The table is flat at y = 0, so the ray's hit on that plane is the exact point.
    const hit = ray.intersectPlane(GROUND, this.hit);
    // A piece beyond the far plane would be clipped, so it could not be seen once placed.
    if (!hit || hit.distanceTo(ray.origin) > this.rig.camera.far) return null;
    return [hit.x, hit.z];
  }

  /** The unit whose top face is under the pointer, or null. */
  private pickUnit(event: PointerEvent): number | null {
    return hitTestUnits(
      this.rayThrough(event),
      this.store.getState().board.pawns,
    );
  }

  /** The camera ray through the pointer's position on the canvas. */
  private rayThrough(event: PointerEvent): Ray {
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.pointer.set(
      ((event.clientX - rect.left) / rect.width) * 2 - 1,
      1 - ((event.clientY - rect.top) / rect.height) * 2,
    );
    this.raycaster.setFromCamera(this.pointer, this.rig.camera);
    return this.raycaster.ray;
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
