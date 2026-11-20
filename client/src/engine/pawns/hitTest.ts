import { type Camera, Plane, type Ray, Vector3 } from "three/webgpu";
import type { Unit } from "../../model";
import type { ScreenBox } from "../coords";
import { BLOCK_DEPTH, BLOCK_THICKNESS, BLOCK_WIDTH } from "./PawnLayer";

/** Every block's top face lies in this plane, since scale leaves thickness unchanged. */
const TOP_FACES = new Plane(new Vector3(0, 1, 0), -BLOCK_THICKNESS);
const hit = new Vector3();
const projected = new Vector3();

/**
 * The id of the unit whose top face the ray hits, or null. Where blocks overlap, the one
 * whose centre is nearest the hit wins.
 */
export function hitTestUnits(ray: Ray, units: readonly Unit[]): number | null {
  if (!ray.intersectPlane(TOP_FACES, hit)) return null;

  let nearest: number | null = null;
  let nearestDistance = Infinity;
  for (const unit of units) {
    const dx = hit.x - unit.pos[0];
    const dz = hit.z - unit.pos[1];
    // Into the unit's frame: undo its rotation around Y.
    const cos = Math.cos(unit.rot);
    const sin = Math.sin(unit.rot);
    const localX = dx * cos - dz * sin;
    const localZ = dx * sin + dz * cos;
    if (
      Math.abs(localX) > (BLOCK_WIDTH / 2) * unit.scale ||
      Math.abs(localZ) > (BLOCK_DEPTH / 2) * unit.scale
    ) {
      continue;
    }
    const distance = dx * dx + dz * dz;
    if (distance < nearestDistance) {
      nearest = unit.id;
      nearestDistance = distance;
    }
  }
  return nearest;
}

/**
 * The ids of the units whose top-face centre lands inside `box` on screen. `canvas` is the
 * canvas's rectangle in the same client coordinates as `box`.
 */
export function unitsInScreenBox(
  units: readonly Unit[],
  camera: Camera,
  canvas: DOMRect,
  box: ScreenBox,
): number[] {
  const ids: number[] = [];
  for (const unit of units) {
    projected.set(unit.pos[0], BLOCK_THICKNESS, unit.pos[1]).project(camera);
    // Behind the camera or beyond the far plane.
    if (projected.z > 1) continue;
    const x = canvas.left + ((projected.x + 1) / 2) * canvas.width;
    const y = canvas.top + ((1 - projected.y) / 2) * canvas.height;
    if (x >= box.left && x <= box.right && y >= box.top && y <= box.bottom) {
      ids.push(unit.id);
    }
  }
  return ids;
}
