import { Euler, MathUtils, PerspectiveCamera, Vector3 } from "three/webgpu";
import type { CameraController } from "./CameraRig";
import { FAST_MULTIPLIER, MOVE_KEYS, type KeyState } from "./KeyState";

/** Movement speed in world units per second, per world unit of height above the table. */
const SPEED_PER_HEIGHT = 0.5;
const MIN_SPEED = 5;
/** Radians of turn per pixel of mouse movement. */
const LOOK_SPEED = 0.0025;
const MAX_PITCH = MathUtils.degToRad(89);
/** Fraction of the camera's height moved per pixel of wheel scroll. */
const WHEEL_SPEED = 0.001;
const MIN_HEIGHT = 1;
const MAX_HEIGHT = 500;

const UP = new Vector3(0, 1, 0);

/**
 * First-person flying, like Tabletop Simulator's camera.
 *
 * - W/A/S/D move horizontally, relative to the direction the camera faces.
 * - R/F move up and down. The `fast` modifier moves faster.
 * - Dragging with the right mouse button held turns the camera.
 * - The mouse wheel moves along the view direction.
 *
 * Movement speeds up with height, so crossing the map takes about as long from any height.
 * Orientation is read back from the camera on every change, so it needs no syncing.
 */
export class FlyController implements CameraController {
  enabled = true;

  private looking = false;
  private euler = new Euler(0, 0, 0, "YXZ");
  private move = new Vector3();
  private forward = new Vector3();

  constructor(
    private camera: PerspectiveCamera,
    private element: HTMLElement,
    private keys: KeyState,
  ) {
    element.addEventListener("pointerdown", this.onPointerDown);
    element.addEventListener("pointermove", this.onPointerMove);
    element.addEventListener("pointerup", this.onPointerUp);
    element.addEventListener("lostpointercapture", this.onLostCapture);
    element.addEventListener("contextmenu", this.onContextMenu);
    element.addEventListener("wheel", this.onWheel, { passive: false });
  }

  sync() {}

  update(dt: number) {
    if (!this.enabled || dt <= 0) return;

    const keys = this.keys;
    const move = this.move.set(
      keys.axis(MOVE_KEYS.left, MOVE_KEYS.right),
      keys.axis(MOVE_KEYS.down, MOVE_KEYS.up),
      keys.axis(MOVE_KEYS.forward, MOVE_KEYS.back),
    );
    if (move.lengthSq() === 0) return;

    // Turn the input by the camera's heading only, so looking down does not fly into the table.
    this.euler.setFromQuaternion(this.camera.quaternion, "YXZ");
    move.normalize().applyAxisAngle(UP, this.euler.y);

    const speed =
      Math.max(MIN_SPEED, this.camera.position.y * SPEED_PER_HEIGHT) *
      (keys.holds("fast") ? FAST_MULTIPLIER : 1);
    this.camera.position.addScaledVector(move, speed * dt);
    this.clampHeight();
  }

  dispose() {
    const element = this.element;
    element.removeEventListener("pointerdown", this.onPointerDown);
    element.removeEventListener("pointermove", this.onPointerMove);
    element.removeEventListener("pointerup", this.onPointerUp);
    element.removeEventListener("lostpointercapture", this.onLostCapture);
    element.removeEventListener("contextmenu", this.onContextMenu);
    element.removeEventListener("wheel", this.onWheel);
  }

  private clampHeight() {
    const position = this.camera.position;
    position.y = MathUtils.clamp(position.y, MIN_HEIGHT, MAX_HEIGHT);
  }

  private onPointerDown = (event: PointerEvent) => {
    if (!this.enabled || event.button !== 2) return;
    this.looking = true;
    this.element.setPointerCapture(event.pointerId);
  };

  private onPointerMove = (event: PointerEvent) => {
    if (!this.looking) return;
    const euler = this.euler.setFromQuaternion(this.camera.quaternion, "YXZ");
    euler.y -= event.movementX * LOOK_SPEED;
    euler.x = MathUtils.clamp(
      euler.x - event.movementY * LOOK_SPEED,
      -MAX_PITCH,
      MAX_PITCH,
    );
    euler.z = 0;
    this.camera.quaternion.setFromEuler(euler);
  };

  private onPointerUp = (event: PointerEvent) => {
    if (event.button !== 2 || !this.looking) return;
    this.element.releasePointerCapture(event.pointerId);
  };

  private onLostCapture = () => {
    this.looking = false;
  };

  private onContextMenu = (event: Event) => {
    event.preventDefault();
  };

  private onWheel = (event: WheelEvent) => {
    if (!this.enabled) return;
    event.preventDefault();
    const pixels =
      event.deltaMode === WheelEvent.DOM_DELTA_LINE
        ? event.deltaY * 16
        : event.deltaY;
    const height = Math.max(this.camera.position.y, MIN_HEIGHT);
    this.camera.getWorldDirection(this.forward);
    this.camera.position.addScaledVector(
      this.forward,
      -pixels * WHEEL_SPEED * height,
    );
    this.clampHeight();
  };
}
