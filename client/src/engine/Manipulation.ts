import { MAX_SCALE, MIN_SCALE, type Unit } from "../model";
import type { Placement } from "../protocol";
import type { TablePoint } from "./coords";
import { previewOf, type PendingPawn } from "./PendingEdits";

/** What a manipulation needs from the engine. */
export interface ManipulationHost {
  preview(edits: Map<number, PendingPawn>): void;
  releasePreview(): void;
  discardPreview(): void;
  /** False when the command was dropped because the connection is down. */
  send(units: Placement[]): boolean;
  /** The manipulation is over, so the next input starts a new one. */
  ended(): void;
}

/**
 * How one input has changed the units so far. Inputs only ever add to the total: the drag's
 * offset, the angle a turn key has turned, the log of the factor a resize key has scaled by.
 * Logs, so that growing and shrinking add up the way angles do.
 */
interface Contribution {
  delta: TablePoint;
  angle: number;
  logScale: number;
}

const NOTHING: ReadonlySet<number> = new Set();

/**
 * One input taking part in a manipulation: a drag, or a held key. It says how far it has
 * changed the units, and when it is done.
 */
export class ManipulationInput {
  /** @internal Read by the manipulation when it adds up its inputs. */
  readonly contribution: Contribution = {
    delta: [0, 0],
    angle: 0,
    logScale: 0,
  };

  constructor(
    private manipulation: Manipulation,
    /** True for an input that holds the units up off the table while it lasts: a drag. */
    readonly lifts: boolean,
  ) {}

  /** Replace this input's part of the change. Fields left out keep their value. */
  set(changes: Partial<Contribution>) {
    Object.assign(this.contribution, changes);
    this.manipulation.show();
  }

  /** This input is done; what it did stays. The last one to finish sends the command. */
  release() {
    this.manipulation.leave(this);
  }

  /** Abandon the whole manipulation, every input's part of it included. */
  cancel() {
    this.manipulation.cancel();
  }
}

/**
 * Units being moved, turned and resized by any number of inputs at once: dragging while
 * holding a turn key, holding turn and resize together, letting go of the mouse and carrying
 * on turning. Each input adds its part to one transform, and one `place` command goes out
 * when the last of them lets go, so the whole manipulation is one undo step.
 *
 * Every frame is worked out from the units as they were when the manipulation began, so the
 * result depends only on the inputs' totals and rounding cannot build up.
 */
export class Manipulation {
  private inputs: ManipulationInput[] = [];
  private active = new Set<ManipulationInput>();
  private over = false;
  private ids: ReadonlySet<number>;
  /** The centroid of the units' positions when the manipulation began. */
  private centroid: TablePoint;

  constructor(
    private host: ManipulationHost,
    /** The units as drawn when the manipulation began. Never empty. */
    private starts: readonly Unit[],
  ) {
    this.ids = new Set(starts.map((unit) => unit.id));
    let cx = 0;
    let cz = 0;
    for (const unit of starts) {
      cx += unit.pos[0] / starts.length;
      cz += unit.pos[1] / starts.length;
    }
    this.centroid = [cx, cz];
  }

  /** True when this unit is one of those being manipulated. */
  has(id: number): boolean {
    return this.ids.has(id);
  }

  /**
   * The units an input is holding up off the table, which is all of them while a drag is part
   * of the manipulation, and none while only keys are turning or resizing them.
   */
  lifted(): ReadonlySet<number> {
    for (const input of this.active) {
      if (input.lifts) return this.ids;
    }
    return NOTHING;
  }

  /**
   * Add an input. Its part of the change starts at nothing. `lifts` is for an input that
   * holds the units up while it lasts, such as a drag.
   */
  join(lifts = false): ManipulationInput {
    const input = new ManipulationInput(this, lifts);
    if (!this.over) {
      this.inputs.push(input);
      this.active.add(input);
    }
    return input;
  }

  /** @internal Draw the units where the inputs have put them so far. */
  show() {
    if (this.over) return;
    this.host.preview(previewOf(this.placements()));
  }

  /** @internal */
  leave(input: ManipulationInput) {
    if (this.over || !this.active.delete(input) || this.active.size > 0) return;
    this.finish();
    if (this.isIdentity()) {
      this.host.discardPreview();
    } else if (this.host.send(this.placements())) {
      this.host.releasePreview();
    } else {
      this.host.discardPreview();
    }
  }

  /** @internal */
  cancel() {
    if (this.over) return;
    this.finish();
    this.host.discardPreview();
  }

  private finish() {
    this.over = true;
    this.active.clear();
    this.host.ended();
  }

  private total(): Contribution {
    const sum: Contribution = { delta: [0, 0], angle: 0, logScale: 0 };
    for (const { contribution: c } of this.inputs) {
      sum.delta[0] += c.delta[0];
      sum.delta[1] += c.delta[1];
      sum.angle += c.angle;
      sum.logScale += c.logScale;
    }
    return sum;
  }

  private isIdentity(): boolean {
    const { delta, angle, logScale } = this.total();
    return delta[0] === 0 && delta[1] === 0 && angle === 0 && logScale === 0;
  }

  /**
   * Where the units go. The formation turns about its centroid, which travels with the drag,
   * so each unit keeps its place in the formation and turns by the same angle. A single unit
   * therefore turns in place. Positive angles are counterclockwise seen from above, as
   * three.js's rotation about +Y; the offset is rotated by the same matrix, so a line of
   * blocks stays lined up as it wheels. Each footprint scales about its own centre, so
   * spacing is unchanged. Only the fields an input has changed are sent.
   */
  private placements(): Placement[] {
    const { delta, angle, logScale } = this.total();
    const [cx, cz] = this.centroid;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    const factor = Math.exp(logScale);
    const moves = angle !== 0 || delta[0] !== 0 || delta[1] !== 0;

    return this.starts.map((unit) => {
      const placement: Placement = { id: unit.id };
      if (moves) {
        const dx = unit.pos[0] - cx;
        const dz = unit.pos[1] - cz;
        placement.pos = [
          cx + delta[0] + dx * cos + dz * sin,
          cz + delta[1] - dx * sin + dz * cos,
        ];
      }
      if (angle !== 0) placement.rot = wrapAngle(unit.rot + angle);
      if (logScale !== 0) {
        placement.scale = Math.min(
          MAX_SCALE,
          Math.max(MIN_SCALE, unit.scale * factor),
        );
      }
      return placement;
    });
  }
}

/** The same angle in [-pi, pi), so rotations stay small however often a unit is turned. */
function wrapAngle(angle: number): number {
  const turn = 2 * Math.PI;
  return angle - turn * Math.floor((angle + Math.PI) / turn);
}
