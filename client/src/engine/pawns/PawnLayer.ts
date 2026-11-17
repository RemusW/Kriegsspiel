import { BoxGeometry, Group, Mesh, MeshStandardMaterial } from "three/webgpu";
import type { Side, Unit } from "../../model";

/** An unscaled unit block: width along X, thickness, depth along Z. */
const BLOCK_WIDTH = 4;
const BLOCK_THICKNESS = 0.5;
const BLOCK_DEPTH = 2;

const SIDE_COLORS: Record<Side, number> = {
  Blue: 0x2f5fb3,
  Red: 0xb8322a,
};

/** Draws the board's units as blocks resting on the map, matched to units by id. */
export class PawnLayer {
  readonly group = new Group();

  private meshes = new Map<number, Mesh>();
  private geometry: BoxGeometry;
  private materials: Record<Side, MeshStandardMaterial>;

  constructor() {
    this.geometry = new BoxGeometry(BLOCK_WIDTH, BLOCK_THICKNESS, BLOCK_DEPTH);
    // Put the origin on the bottom face, so a block at y = 0 rests on the map.
    this.geometry.translate(0, BLOCK_THICKNESS / 2, 0);
    this.materials = {
      Blue: new MeshStandardMaterial({ color: SIDE_COLORS.Blue }),
      Red: new MeshStandardMaterial({ color: SIDE_COLORS.Red }),
    };
  }

  /** Add, update and remove meshes so they match these units. */
  sync(units: Unit[]) {
    const present = new Set<number>();
    for (const unit of units) {
      present.add(unit.id);
      let mesh = this.meshes.get(unit.id);
      if (!mesh) {
        mesh = new Mesh(this.geometry);
        this.meshes.set(unit.id, mesh);
        this.group.add(mesh);
      }
      mesh.material = this.materials[unit.side];
      mesh.position.set(unit.pos[0], 0, unit.pos[1]);
      mesh.rotation.y = unit.rot;
      // Scale changes the footprint only; every block keeps the same thickness.
      mesh.scale.set(unit.scale, 1, unit.scale);
    }
    for (const [id, mesh] of this.meshes) {
      if (!present.has(id)) {
        this.group.remove(mesh);
        this.meshes.delete(id);
      }
    }
  }

  dispose() {
    this.geometry.dispose();
    this.materials.Blue.dispose();
    this.materials.Red.dispose();
  }
}
