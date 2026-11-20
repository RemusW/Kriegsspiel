import { Plane, Raycaster, Vector2, Vector3, type Ray } from "three/webgpu";
import type { Unit } from "../../model";
import type { CameraRig } from "../camera/CameraRig";
import type { ScreenBox, ScreenPoint, TablePoint } from "../coords";
import { hitTestUnits, unitsInScreenBox } from "../pawns/hitTest";

/** The table: the plane the map and every piece rest on. */
const GROUND = new Plane(new Vector3(0, 1, 0), 0);

/** What tools and actions may ask about the scene under a point on the canvas. */
export interface PickSource {
  /** The point on the table under this point as [world X, world Z], on or off the map. */
  table(point: ScreenPoint): TablePoint | null;
  /** The unit whose top face is under this point, or null. */
  unit(point: ScreenPoint): number | null;
  /** The units whose top-face centre is inside this box on screen. */
  unitsInBox(box: ScreenBox): number[];
}

/**
 * Turns points on the canvas into points on the table and the units under them. It picks
 * against the units as they are drawn rather than the stored board, so a unit being dragged
 * is picked where the umpire sees it.
 */
export class Picker implements PickSource {
  private raycaster = new Raycaster();
  private pointer = new Vector2();
  private hit = new Vector3();

  constructor(
    private canvas: HTMLCanvasElement,
    private rig: CameraRig,
    /** The units as drawn. Read on every pick: boards arrive and drags move units. */
    private units: () => readonly Unit[],
  ) {}

  /**
   * Null when the point is above the horizon or the table there is beyond the camera's far
   * plane, where a piece would be clipped and so could not be seen once placed.
   */
  table(point: ScreenPoint): TablePoint | null {
    const ray = this.ray(point);
    // The table is flat at y = 0, so the ray's hit on that plane is the exact point.
    const hit = ray.intersectPlane(GROUND, this.hit);
    if (!hit || hit.distanceTo(ray.origin) > this.rig.camera.far) return null;
    return [hit.x, hit.z];
  }

  unit(point: ScreenPoint): number | null {
    return hitTestUnits(this.ray(point), this.units());
  }

  unitsInBox(box: ScreenBox): number[] {
    return unitsInScreenBox(
      this.units(),
      this.rig.camera,
      this.canvas.getBoundingClientRect(),
      box,
    );
  }

  /** The camera ray through this point on the canvas. */
  ray(point: ScreenPoint): Ray {
    const rect = this.canvas.getBoundingClientRect();
    this.pointer.set(
      ((point.x - rect.left) / rect.width) * 2 - 1,
      1 - ((point.y - rect.top) / rect.height) * 2,
    );
    this.raycaster.setFromCamera(this.pointer, this.rig.camera);
    return this.raycaster.ray;
  }
}
