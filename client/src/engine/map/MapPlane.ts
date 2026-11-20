import {
  Mesh,
  MeshStandardMaterial,
  PlaneGeometry,
  SRGBColorSpace,
  type Texture,
  TextureLoader,
} from "three/webgpu";
import { assetUrl } from "../../net/assets";

/** Width of the map plane in world units. Its depth follows the image's aspect ratio. */
export const MAP_WIDTH = 100;
/** Depth of the blank default map, shown until an umpire lays an image down. */
export const BLANK_MAP_DEPTH = 70;
/** The bare table, where no map image has been laid. */
const BLANK_COLOUR = 0x3c4049;

/**
 * The table top: one subdivided plane, with the board's map image on it when it has one.
 *
 * It is given an asset id and fetches the image itself, by URL rather than through the
 * socket, so the browser caches it: an id is the hash of the image's bytes, so the URL can
 * never come to mean something else. The plane's depth follows the image's aspect ratio,
 * which is why the owner of the geometry is also the owner of the texture.
 */
export class MapPlane {
  /** Depth of the plane in world units, for framing the camera on the table. */
  depth = BLANK_MAP_DEPTH;

  private mesh: Mesh;
  private material: MeshStandardMaterial;
  private texture: Texture | null = null;
  /** The image asked for most recently, which a load that finishes late is checked against. */
  private wanted: string | null = null;
  private disposed = false;

  constructor(private anisotropy: () => number) {
    this.material = new MeshStandardMaterial({
      color: BLANK_COLOUR,
      roughness: 1,
    });
    this.mesh = new Mesh(plane(BLANK_MAP_DEPTH), this.material);
  }

  get object(): Mesh {
    return this.mesh;
  }

  /**
   * Show this image, or the blank table with null. Returns once the plane has been rebuilt,
   * with the new depth. Two changes in quick succession can finish out of order, so a load
   * that is no longer the one wanted is thrown away rather than drawn, and one that fails is
   * only reported if it is still wanted.
   */
  async show(id: string | null): Promise<number> {
    this.wanted = id;
    let texture: Texture | null = null;
    try {
      if (id !== null) {
        texture = await new TextureLoader().loadAsync(assetUrl(id));
      }
    } catch (error) {
      if (this.disposed || this.wanted !== id) return this.depth;
      throw error;
    }
    if (this.disposed || this.wanted !== id) {
      texture?.dispose();
      return this.depth;
    }

    if (texture) {
      texture.colorSpace = SRGBColorSpace;
      texture.anisotropy = this.anisotropy();
      const image = texture.image as { width: number; height: number };
      this.depth = MAP_WIDTH * (image.height / image.width);
    } else {
      this.depth = BLANK_MAP_DEPTH;
    }

    this.texture?.dispose();
    this.texture = texture;
    this.material.map = texture;
    // A map going on or coming off changes whether the plane's own colour shows through.
    this.material.color.set(texture ? 0xffffff : BLANK_COLOUR);
    this.material.needsUpdate = true;

    this.mesh.geometry.dispose();
    this.mesh.geometry = plane(this.depth);
    return this.depth;
  }

  dispose() {
    this.disposed = true;
    this.mesh.geometry.dispose();
    this.material.dispose();
    this.texture?.dispose();
  }
}

/** One quad per world unit, rotated onto the XZ plane so the top of the image faces -Z. */
function plane(depth: number): PlaneGeometry {
  const geometry = new PlaneGeometry(
    MAP_WIDTH,
    depth,
    MAP_WIDTH,
    Math.max(1, Math.round(depth)),
  );
  geometry.rotateX(-Math.PI / 2);
  return geometry;
}
