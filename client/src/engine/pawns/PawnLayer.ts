import {
  BoxGeometry,
  Group,
  Mesh,
  type MeshBasicNodeMaterial,
  MeshStandardMaterial,
} from "three/webgpu";
import type { Side, Unit } from "../../model";
import { OVERLAY_LAYER } from "../layers";
import {
  addOutlineNormals,
  createOutlineMaterial,
  maskOutlines,
  OUTLINE_RENDER_ORDER,
} from "../materials/outline";
import {
  SPRING_STEP,
  springAt,
  stepSpring,
  type SpringState,
  type SpringTune,
} from "./spring";

/** An unscaled unit block: width along X, thickness, depth along Z. */
export const BLOCK_WIDTH = 4;
export const BLOCK_THICKNESS = 0.5;
export const BLOCK_DEPTH = 2;

const SIDE_COLORS: Record<Side, number> = {
  Blue: 0x2f5fb3,
  Red: 0xb8322a,
};

const SELECTION_COLOR = 0xffd400;
/** Selection outline width in CSS pixels. */
const SELECTION_WIDTH = 2;

/** How a block follows its unit across the table: quick, with only a trace of overshoot. */
const FOLLOW: SpringTune = { frequency: 9, damping: 0.85 };
/** How it turns and resizes to match. */
const TURN: SpringTune = { frequency: 8, damping: 0.8 };
const RESIZE: SpringTune = { frequency: 8, damping: 0.7 };
/** Picking a block up: soft enough to overshoot a little. Letting go drops it under GRAVITY. */
const LIFT: SpringTune = { frequency: 6, damping: 0.6 };
/** Leaning into the direction it is carried, and swinging back when it stops. */
const TILT: SpringTune = { frequency: 5, damping: 0.45 };
/** The squash on landing: a quick, lightly damped wobble. */
const SQUASH: SpringTune = { frequency: 10, damping: 0.35 };

/** How high a held block is carried above the table, in world units. */
const LIFT_HEIGHT = 0.8;
/** How high a new unit appears before it drops onto the table. */
const SPAWN_HEIGHT = 3;
/** Radians of lean per world unit a second of movement, and the most it leans. */
const TILT_PER_SPEED = 0.012;
const MAX_TILT = 0.3;
/**
 * How fast a dropped block falls, in world units a second squared. Stronger than real
 * gravity at this scale, so a drop from the carrying height lands in about a fifth of a
 * second.
 */
const GRAVITY = 40;
/** The share of its speed a block keeps when it bounces off the table. */
const BOUNCE = 0.35;
/** Landing slower than this, in world units a second, does not bounce or squash. */
const MIN_IMPACT = 0.5;
/** How far a landing squashes the block, per world unit a second of impact speed. */
const SQUASH_PER_SPEED = 0.6;

/** Where a unit is and how its block is moving towards it. */
interface PawnMotion {
  /** Where the unit is: what the board, with pending edits, says. */
  target: Pick<Unit, "pos" | "rot" | "scale">;
  x: SpringState;
  z: SpringState;
  yaw: SpringState;
  scale: SpringState;
  /** Height above the table. */
  lift: SpringState;
  /** Lean around world X and world Z, in radians. */
  tiltX: SpringState;
  tiltZ: SpringState;
  /** Thickness relative to normal: below 1 while squashed by a landing. */
  squash: SpringState;
}

/** One unit's meshes. The outline is a child of the block, so it follows its transform. */
interface PawnView {
  block: Mesh;
  outline: Mesh;
  motion: PawnMotion;
}

/**
 * Draws the board's units as blocks on the map, matched to units by id. Each block follows
 * its unit on springs rather than jumping to it: it slides to a new position, turns and
 * resizes smoothly, is lifted and leans while held, and drops with a small bounce when let
 * go. This is drawing only. The units' positions, which picking and every edit use, are the
 * targets, so a block may trail its unit for a moment but never changes where the unit is.
 */
export class PawnLayer {
  readonly group = new Group();

  private views = new Map<number, PawnView>();
  private geometry: BoxGeometry;
  private materials: Record<Side, MeshStandardMaterial>;
  private outlineMaterial: MeshBasicNodeMaterial;
  /** Time not yet stepped, carried to the next frame so the springs keep a fixed step. */
  private unstepped = 0;
  /**
   * Set once a board with units has been drawn. Units already there when the client joins are
   * placed at once; units that appear after that drop onto the table.
   */
  private seeded = false;

  constructor() {
    this.geometry = new BoxGeometry(BLOCK_WIDTH, BLOCK_THICKNESS, BLOCK_DEPTH);
    // Put the origin on the bottom face, so a block at y = 0 rests on the map.
    this.geometry.translate(0, BLOCK_THICKNESS / 2, 0);
    addOutlineNormals(this.geometry);
    this.materials = {
      Blue: new MeshStandardMaterial({ color: SIDE_COLORS.Blue }),
      Red: new MeshStandardMaterial({ color: SIDE_COLORS.Red }),
    };
    // Blocks hide outlines: a selected unit's outline stops at any block in front of it.
    maskOutlines(this.materials.Blue);
    maskOutlines(this.materials.Red);
    this.outlineMaterial = createOutlineMaterial(
      SELECTION_COLOR,
      SELECTION_WIDTH,
    );
  }

  /**
   * Add, update and remove meshes so they match these units. The engine calls this with the
   * units as drawn, so a drag in progress arrives here as ordinary positions.
   */
  sync(units: readonly Unit[]) {
    const present = new Set<number>();
    for (const unit of units) {
      present.add(unit.id);
      let view = this.views.get(unit.id);
      if (!view) {
        view = this.createView(unit, this.seeded ? SPAWN_HEIGHT : 0);
        this.views.set(unit.id, view);
        this.group.add(view.block);
        this.place(view);
      }
      view.block.material = this.materials[unit.side];
      view.motion.target = unit;
    }
    if (units.length > 0) this.seeded = true;
    for (const [id, view] of this.views) {
      if (!present.has(id)) {
        this.group.remove(view.block);
        this.views.delete(id);
      }
    }
  }

  /**
   * Move every block on towards its unit by `dt` seconds. `lifted` are the units being held,
   * which are carried above the table; the rest rest on it. Left out, none are held.
   */
  update(dt: number, lifted?: ReadonlySet<number>) {
    this.unstepped += dt;
    while (this.unstepped >= SPRING_STEP) {
      this.unstepped -= SPRING_STEP;
      for (const [id, view] of this.views) {
        step(view.motion, lifted?.has(id) ?? false, SPRING_STEP);
      }
    }
    for (const view of this.views.values()) this.place(view);
  }

  /** Outline the units with these ids, and no others. */
  showSelection(ids: readonly number[]) {
    const selected = new Set(ids);
    for (const [id, view] of this.views) {
      view.outline.visible = selected.has(id);
    }
  }

  dispose() {
    this.geometry.dispose();
    this.materials.Blue.dispose();
    this.materials.Red.dispose();
    this.outlineMaterial.dispose();
  }

  /** Put the block where its motion says it is now. */
  private place({ block, motion }: PawnView) {
    block.position.set(motion.x.value, motion.lift.value, motion.z.value);
    // Turn the block about its own axis first, then lean it in world space, so it leans the
    // way it moves whichever way it faces.
    block.rotation.set(
      motion.tiltX.value,
      motion.yaw.value,
      motion.tiltZ.value,
    );
    // A squashed block keeps roughly its volume, so it spreads as it flattens. Scale changes
    // the footprint only; every block keeps the same thickness.
    const squash = motion.squash.value;
    const spread = motion.scale.value / Math.sqrt(squash);
    block.scale.set(spread, squash, spread);
  }

  private createView(unit: Unit, height: number): PawnView {
    const block = new Mesh(this.geometry);
    // Euler "ZXY" applies Y (the unit's own rotation) first and the X and Z lean after it,
    // about world axes.
    block.rotation.order = "ZXY";
    const outline = new Mesh(this.geometry, this.outlineMaterial);
    outline.layers.set(OVERLAY_LAYER);
    outline.renderOrder = OUTLINE_RENDER_ORDER;
    outline.visible = false;
    block.add(outline);
    const motion: PawnMotion = {
      target: unit,
      x: springAt(unit.pos[0]),
      z: springAt(unit.pos[1]),
      yaw: springAt(unit.rot),
      scale: springAt(unit.scale),
      lift: springAt(height),
      tiltX: springAt(0),
      tiltZ: springAt(0),
      squash: springAt(1),
    };
    return { block, outline, motion };
  }
}

/** Advance one block's springs by `dt` seconds. */
function step(motion: PawnMotion, lifted: boolean, dt: number) {
  const { target } = motion;
  stepSpring(motion.x, target.pos[0], FOLLOW, dt);
  stepSpring(motion.z, target.pos[1], FOLLOW, dt);
  // Turn the short way round: rotations are kept in [-pi, pi), so a unit turned from just
  // below pi to just above -pi must not spin all the way back.
  const yaw = motion.yaw.value;
  stepSpring(motion.yaw, yaw + shortestTurn(target.rot - yaw), TURN, dt);
  stepSpring(motion.scale, target.scale, RESIZE, dt);

  // A held block hangs from the pointer like a weight, so its bottom trails behind: moving
  // towards +X leans it about Z one way, moving towards +Z leans it about X the other.
  const leanX = lifted ? clampLean(motion.z.velocity * TILT_PER_SPEED) : 0;
  const leanZ = lifted ? clampLean(-motion.x.velocity * TILT_PER_SPEED) : 0;
  stepSpring(motion.tiltX, leanX, TILT, dt);
  stepSpring(motion.tiltZ, leanZ, TILT, dt);

  // Held, a spring carries the block up to the carrying height. Let go, it falls: a spring
  // would also pull it down, but its damping would brake the fall and leave no bounce.
  const lift = motion.lift;
  if (lifted) {
    stepSpring(lift, LIFT_HEIGHT, LIFT, dt);
  } else {
    lift.velocity -= GRAVITY * dt;
    lift.value += lift.velocity * dt;
  }
  if (lift.value < 0) {
    // The table stops it. A hard landing bounces and squashes the block; a soft one settles.
    lift.value = 0;
    const impact = -lift.velocity;
    if (impact > MIN_IMPACT) {
      lift.velocity = impact * BOUNCE;
      motion.squash.velocity -= impact * SQUASH_PER_SPEED;
    } else {
      lift.velocity = 0;
    }
  }
  stepSpring(motion.squash, 1, SQUASH, dt);
}

/** The same turn as `angle`, as at most half a turn either way. */
function shortestTurn(angle: number): number {
  const turn = 2 * Math.PI;
  return angle - turn * Math.round(angle / turn);
}

function clampLean(lean: number): number {
  return Math.max(-MAX_TILT, Math.min(MAX_TILT, lean));
}
