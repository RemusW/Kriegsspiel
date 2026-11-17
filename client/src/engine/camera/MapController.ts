import { MapControls } from "three/addons/controls/MapControls.js";
import {
  MathUtils,
  MOUSE,
  PerspectiveCamera,
  Plane,
  Ray,
  Vector3,
} from "three/webgpu";
import type { CameraController } from "./CameraRig";
import { FAST_MULTIPLIER, MOVE_KEYS, type KeyState } from "./KeyState";

/** Lowest the camera may sink towards the horizon, measured up from the table. */
const MIN_PITCH = MathUtils.degToRad(30);
const MIN_DISTANCE = 10;
const MAX_DISTANCE = 400;
/** Keyboard panning speed in world units per second, per world unit of distance to the target. */
const PAN_SPEED_PER_DISTANCE = 0.5;

const GROUND = new Plane(new Vector3(0, 1, 0), 0);

/**
 * A map-style camera that orbits a point on the table, with north kept at the top of the
 * screen.
 *
 * - Dragging with the right mouse button held pans; the table point under the cursor stays
 *   under it.
 * - Dragging with the middle button held tilts the camera.
 * - The mouse wheel zooms towards the cursor.
 * - W/A/S/D pan. Shift pans faster.
 */
export class MapController implements CameraController {
  private controls: MapControls;
  private ray = new Ray();
  private direction = new Vector3();
  private move = new Vector3();

  constructor(
    private camera: PerspectiveCamera,
    element: HTMLElement,
    private keys: KeyState,
  ) {
    const controls = new MapControls(camera, element);
    controls.mouseButtons = { MIDDLE: MOUSE.ROTATE, RIGHT: MOUSE.PAN };
    controls.maxPolarAngle = Math.PI / 2 - MIN_PITCH;
    controls.minAzimuthAngle = 0;
    controls.maxAzimuthAngle = 0;
    controls.minDistance = MIN_DISTANCE;
    controls.maxDistance = MAX_DISTANCE;
    controls.zoomToCursor = true;
    controls.enableDamping = true;
    this.controls = controls;
    this.sync();
  }

  get enabled(): boolean {
    return this.controls.enabled;
  }

  set enabled(value: boolean) {
    this.controls.enabled = value;
  }

  /**
   * Orbit the table point at the centre of the view. When the camera looks at or above the
   * horizon, orbit the table point ahead of it instead, as far ahead as the camera is high.
   */
  sync() {
    const target = this.controls.target;
    this.camera.getWorldDirection(this.direction);
    this.ray.set(this.camera.position, this.direction);
    if (!this.ray.intersectPlane(GROUND, target)) {
      this.direction.setY(0).normalize();
      target
        .copy(this.camera.position)
        .addScaledVector(this.direction, this.camera.position.y)
        .setY(0);
    }
    this.controls.update();
  }

  update(dt: number) {
    if (this.controls.enabled && dt > 0) this.panWithKeys(dt);
    // Every frame, so damping can finish easing out a drag.
    this.controls.update();
  }

  dispose() {
    this.controls.dispose();
  }

  private panWithKeys(dt: number) {
    const keys = this.keys;
    const right = keys.axis(MOVE_KEYS.left, MOVE_KEYS.right);
    const back = keys.axis(MOVE_KEYS.forward, MOVE_KEYS.back);
    if (right === 0 && back === 0) return;

    // Screen right and screen down, along the table, for the current heading.
    const azimuth = this.controls.getAzimuthalAngle();
    const move = this.move
      .set(
        right * Math.cos(azimuth) + back * Math.sin(azimuth),
        0,
        -right * Math.sin(azimuth) + back * Math.cos(azimuth),
      )
      .normalize();

    const distance = this.camera.position.distanceTo(this.controls.target);
    const speed =
      distance * PAN_SPEED_PER_DISTANCE * (keys.shift ? FAST_MULTIPLIER : 1);
    move.multiplyScalar(speed * dt);
    this.camera.position.add(move);
    this.controls.target.add(move);
  }
}
