import {
  Group,
  Mesh,
  MeshStandardMaterial,
  PlaneGeometry,
  SRGBColorSpace,
  type Texture,
  TextureLoader,
} from "three/webgpu";
import type { MapImage } from "../../model";
import { assetUrl } from "../../net/assets";
import type { TableArea } from "../coords";

/**
 * The bare table, shown while no map has been laid. Its width is the width a map image is
 * given when it goes down, which the server decides (`NEW_MAP_WIDTH` in
 * `crates/server/src/model.rs`), so an empty table is the size of the first sheet laid on it.
 */
export const BLANK_TABLE: TableArea = {
  centre: [0, 0],
  width: 100,
  depth: 70,
};

/** The bare table, where no map has been laid, and a sheet whose image is still loading. */
const BLANK_COLOUR = 0x3c4049;

/**
 * Vertical gap between stacked sheets. Maps are flat on the table and may overlap, and two
 * coplanar quads have no settled answer to which is in front, so each sheet sits this much
 * above the one below it. Far too small to see a piece sink into, at the scale of a block
 * half a unit thick.
 */
const SHEET_LIFT = 0.005;

/** One sheet on the table: its mesh, and the image it is currently showing. */
interface Sheet {
  mesh: Mesh;
  material: MeshStandardMaterial;
  texture: Texture | null;
  /** The map as this sheet is drawn, to compare the board against. */
  shown: MapImage;
  /** The image asked for most recently, which a load that finishes late is checked against. */
  wanted: string | null;
}

/**
 * The table top: one sheet per map on the board, and a blank plane while there are none.
 *
 * Each sheet's extent comes from the board rather than from its image, so the geometry is
 * right before the image has loaded and every client agrees on where a map's edges are. The
 * image itself is fetched by URL rather than through the socket, so the browser caches it: an
 * id is the hash of the image's bytes, so the URL can never come to mean something else.
 */
export class MapTable {
  /** Everything the table draws. Added to the scene once. */
  readonly group = new Group();

  private sheets = new Map<number, Sheet>();
  private blank: Mesh;
  private blankMaterial: MeshStandardMaterial;
  /** The maps as last synced, for the area the camera is framed on. */
  private maps: readonly MapImage[] = [];
  private disposed = false;

  constructor(
    private anisotropy: () => number,
    /** Told about an image that will not load, which no board can show by itself. */
    private onError: (asset: string, error: unknown) => void,
  ) {
    this.blankMaterial = new MeshStandardMaterial({
      color: BLANK_COLOUR,
      roughness: 1,
    });
    this.blank = new Mesh(
      sheetGeometry(BLANK_TABLE.width, BLANK_TABLE.depth),
      this.blankMaterial,
    );
    this.group.add(this.blank);
  }

  /**
   * Draw these maps. Called with every board, so it only does the work the board asks for: a
   * sheet is kept by its map's id, its geometry is rebuilt when its extent changes, and its
   * image is fetched only when the map names a different one.
   */
  sync(maps: readonly MapImage[]) {
    const ids = new Set(maps.map((map) => map.id));
    for (const [id, sheet] of this.sheets) {
      if (!ids.has(id)) {
        this.sheets.delete(id);
        this.discard(sheet);
      }
    }

    maps.forEach((map, index) => {
      const sheet = this.sheets.get(map.id) ?? this.add(map);
      // The board's order is the drawing order: each sheet sits above the one before it.
      sheet.mesh.position.set(map.pos[0], index * SHEET_LIFT, map.pos[1]);
      const [width, depth] = sheet.shown.size;
      if (map.size[0] !== width || map.size[1] !== depth) {
        sheet.mesh.geometry.dispose();
        sheet.mesh.geometry = sheetGeometry(map.size[0], map.size[1]);
      }
      if (map.asset !== sheet.wanted) void this.load(sheet, map.asset);
      sheet.shown = map;
    });

    this.maps = maps;
    this.blank.visible = maps.length === 0;
  }

  /**
   * The table to frame the camera on: everything the maps cover, or the blank table while
   * none are down.
   */
  area(): TableArea {
    if (this.maps.length === 0) return BLANK_TABLE;
    let left = Infinity;
    let right = -Infinity;
    let near = Infinity;
    let far = -Infinity;
    for (const map of this.maps) {
      left = Math.min(left, map.pos[0] - map.size[0] / 2);
      right = Math.max(right, map.pos[0] + map.size[0] / 2);
      near = Math.min(near, map.pos[1] - map.size[1] / 2);
      far = Math.max(far, map.pos[1] + map.size[1] / 2);
    }
    return {
      centre: [(left + right) / 2, (near + far) / 2],
      width: right - left,
      depth: far - near,
    };
  }

  dispose() {
    this.disposed = true;
    for (const sheet of this.sheets.values()) this.discard(sheet);
    this.sheets.clear();
    this.blank.geometry.dispose();
    this.blankMaterial.dispose();
  }

  /** A sheet with no image yet: the right shape and size, in the bare table's colour. */
  private add(map: MapImage): Sheet {
    const material = new MeshStandardMaterial({
      color: BLANK_COLOUR,
      roughness: 1,
    });
    const mesh = new Mesh(sheetGeometry(map.size[0], map.size[1]), material);
    this.group.add(mesh);
    const sheet: Sheet = {
      mesh,
      material,
      texture: null,
      shown: map,
      wanted: null,
    };
    this.sheets.set(map.id, sheet);
    return sheet;
  }

  /**
   * Fetch this sheet's image and put it on. Two changes in quick succession can finish out
   * of order, so a load that is no longer the one wanted is thrown away rather than drawn,
   * and one that fails is only reported if it is still wanted.
   */
  private async load(sheet: Sheet, asset: string) {
    sheet.wanted = asset;
    let texture: Texture;
    try {
      texture = await new TextureLoader().loadAsync(assetUrl(asset));
    } catch (error) {
      if (!this.disposed && sheet.wanted === asset) this.onError(asset, error);
      return;
    }
    if (this.disposed || sheet.wanted !== asset) {
      texture.dispose();
      return;
    }

    texture.colorSpace = SRGBColorSpace;
    texture.anisotropy = this.anisotropy();
    sheet.texture?.dispose();
    sheet.texture = texture;
    sheet.material.map = texture;
    // The sheet's own colour showed through while it was loading.
    sheet.material.color.set(0xffffff);
    sheet.material.needsUpdate = true;
  }

  private discard(sheet: Sheet) {
    this.group.remove(sheet.mesh);
    sheet.mesh.geometry.dispose();
    sheet.material.dispose();
    sheet.texture?.dispose();
    // Nothing may draw with this sheet again, so a load still in flight has to be dropped.
    sheet.wanted = null;
  }
}

/** One quad per world unit, rotated onto the XZ plane so the top of the image faces -Z. */
function sheetGeometry(width: number, depth: number): PlaneGeometry {
  const geometry = new PlaneGeometry(
    width,
    depth,
    Math.max(1, Math.round(width)),
    Math.max(1, Math.round(depth)),
  );
  geometry.rotateX(-Math.PI / 2);
  return geometry;
}
