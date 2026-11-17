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

/** One unit's meshes. The outline is a child of the block, so it follows its transform. */
interface PawnView {
  block: Mesh;
  outline: Mesh;
}

/** Draws the board's units as blocks resting on the map, matched to units by id. */
export class PawnLayer {
  readonly group = new Group();

  private views = new Map<number, PawnView>();
  private geometry: BoxGeometry;
  private materials: Record<Side, MeshStandardMaterial>;
  private outlineMaterial: MeshBasicNodeMaterial;

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

  /** Add, update and remove meshes so they match these units. */
  sync(units: Unit[]) {
    const present = new Set<number>();
    for (const unit of units) {
      present.add(unit.id);
      let view = this.views.get(unit.id);
      if (!view) {
        view = this.createView();
        this.views.set(unit.id, view);
        this.group.add(view.block);
      }
      const block = view.block;
      block.material = this.materials[unit.side];
      block.position.set(unit.pos[0], 0, unit.pos[1]);
      block.rotation.y = unit.rot;
      // Scale changes the footprint only; every block keeps the same thickness.
      block.scale.set(unit.scale, 1, unit.scale);
    }
    for (const [id, view] of this.views) {
      if (!present.has(id)) {
        this.group.remove(view.block);
        this.views.delete(id);
      }
    }
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

  private createView(): PawnView {
    const block = new Mesh(this.geometry);
    const outline = new Mesh(this.geometry, this.outlineMaterial);
    outline.layers.set(OVERLAY_LAYER);
    outline.renderOrder = OUTLINE_RENDER_ORDER;
    outline.visible = false;
    block.add(outline);
    return { block, outline };
  }
}
