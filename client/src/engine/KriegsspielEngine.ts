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
import { Unit } from "../model";
import type { ClientStore } from "../state/store";
import { CameraRig } from "./camera/CameraRig";

/** Width of the map plane in world units. Its depth follows the image's aspect ratio. */
const MAP_WIDTH = 100;
/** Depth of the blank default map, framed until a map image has loaded. */
const BLANK_MAP_DEPTH = 70;
/** Longest frame time the camera moves by, so a stalled or hidden tab does not cause a jump. */
const MAX_FRAME_SECONDS = 0.1;

export class KSEngine {
  private renderer: WebGPURenderer;
  private rig: CameraRig;
  private scene: Scene;

  private resizeObserver: ResizeObserver;
  private unsubscribe: () => void;
  private disposed = false;
  private lastFrameTime: number | null = null;

  constructor(canvas: HTMLCanvasElement, store: ClientStore) {
    this.renderer = new WebGPURenderer({ canvas: canvas, antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setAnimationLoop((time) => this.render(time));
    this.rig = new CameraRig(canvas, store.getState().cameraMode);
    this.rig.frameMap(MAP_WIDTH, BLANK_MAP_DEPTH);
    this.unsubscribe = store.subscribe((state, previous) => {
      if (state.cameraMode !== previous.cameraMode) {
        this.rig.setMode(state.cameraMode);
      }
    });

    this.scene = new Scene();

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
    this.rig.dispose();
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

  addUnit() {
    let unit: Unit;
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
