import { MathUtils, PerspectiveCamera } from "three/webgpu";
import type { CameraMode } from "../../state/store";
import type { Keymap } from "../input/Keymap";
import { FlyController } from "./FlyController";
import { KeyState } from "./KeyState";
import { MapController } from "./MapController";

/** Vertical field of view in degrees. */
const FOV = 45;
const NEAR = 0.1;
const FAR = 1000;

/** How far below the horizon the camera looks when framing a map. */
const FRAME_PITCH = MathUtils.degToRad(50);
/** Framing distance from the map's centre, as a multiple of its longer side. */
const FRAME_DISTANCE = 1.4;

/** One way of steering the camera from mouse and keyboard input. */
export interface CameraController {
  enabled: boolean;
  /** Move the camera for this frame. `dt` is the frame time in seconds. */
  update(dt: number): void;
  /** Continue from wherever the camera now is, after code outside the controller moved it. */
  sync(): void;
  /** Remove the controller's event listeners. */
  dispose(): void;
}

/** The camera over the table, steered by the controller for the current mode. */
export class CameraRig {
  /** The three.js camera to render with. */
  readonly camera = new PerspectiveCamera(FOV, 1, NEAR, FAR);

  private keys: KeyState;
  private mode: CameraMode;
  private controller: CameraController;
  private isEnabled = true;

  /** `element` receives the mouse input; keys are read from the whole window. */
  constructor(
    private element: HTMLElement,
    mode: CameraMode,
    keymap: Keymap,
  ) {
    this.keys = new KeyState(keymap);
    this.mode = mode;
    this.controller = this.createController(mode);
  }

  get enabled(): boolean {
    return this.isEnabled;
  }

  set enabled(value: boolean) {
    this.isEnabled = value;
    this.controller.enabled = value;
  }

  /** Switch controllers. The camera keeps its place; the new controller continues from it. */
  setMode(mode: CameraMode) {
    if (mode === this.mode) return;
    this.controller.dispose();
    this.mode = mode;
    this.controller = this.createController(mode);
    this.controller.enabled = this.isEnabled;
  }

  /** Match the projection to a drawing area of this size, in pixels. */
  setSize(width: number, height: number) {
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
  }

  /**
   * Place the camera south of a map centred on the origin, looking north at its centre, far
   * enough back that the whole map is in view.
   */
  frameMap(width: number, depth: number) {
    const distance = FRAME_DISTANCE * Math.max(width, depth);
    this.camera.position.set(
      0,
      distance * Math.sin(FRAME_PITCH),
      distance * Math.cos(FRAME_PITCH),
    );
    this.camera.lookAt(0, 0, 0);
    this.controller.sync();
  }

  update(dt: number) {
    this.controller.update(dt);
  }

  dispose() {
    this.controller.dispose();
    this.keys.dispose();
  }

  private createController(mode: CameraMode): CameraController {
    switch (mode) {
      case "fly":
        return new FlyController(this.camera, this.element, this.keys);
      case "map":
        return new MapController(this.camera, this.element, this.keys);
    }
  }
}
