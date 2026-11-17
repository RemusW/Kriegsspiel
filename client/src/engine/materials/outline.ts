import {
  AlwaysStencilFunc,
  BackSide,
  BufferAttribute,
  type BufferGeometry,
  type ColorRepresentation,
  type Material,
  MeshBasicNodeMaterial,
  NotEqualStencilFunc,
  ReplaceStencilOp,
  Vector3,
} from "three/webgpu";
import {
  attribute,
  cameraProjectionMatrix,
  Fn,
  max,
  modelViewMatrix,
  positionLocal,
  screenSize,
  uniform,
  vec4,
} from "three/tsl";

/** Stencil value written by meshes that outlines must not cover. */
const OUTLINE_MASK = 1;

/**
 * Outline meshes need a higher `renderOrder` than every mesh that masks them, so the mask is
 * in the stencil buffer before they draw.
 */
export const OUTLINE_RENDER_ORDER = 1;

/**
 * Keep outlines off a material's meshes: wherever they are visible, they mark the stencil
 * buffer, and outlines skip marked pixels. Needs a renderer created with `stencil: true`.
 */
export function maskOutlines(material: Material) {
  material.stencilWrite = true;
  material.stencilFunc = AlwaysStencilFunc;
  material.stencilRef = OUTLINE_MASK;
  material.stencilZPass = ReplaceStencilOp;
}

/**
 * Give a geometry the `outlineNormal` attribute that outline materials push vertices along:
 * at each vertex, the normalised sum of the normals of every vertex at the same position.
 * Hard-edged geometry such as a box has a separate vertex per face at each corner. Pushed out
 * along their own face normals, those would tear the outline open at the corners.
 */
export function addOutlineNormals(geometry: BufferGeometry) {
  const position = geometry.getAttribute("position");
  const normal = geometry.getAttribute("normal");
  const key = (i: number) =>
    `${position.getX(i).toFixed(4)},${position.getY(i).toFixed(4)},${position.getZ(i).toFixed(4)}`;

  const sums = new Map<string, Vector3>();
  const n = new Vector3();
  for (let i = 0; i < position.count; i++) {
    const k = key(i);
    let sum = sums.get(k);
    if (!sum) {
      sum = new Vector3();
      sums.set(k, sum);
    }
    sum.add(n.fromBufferAttribute(normal, i));
  }

  const out = new Float32Array(position.count * 3);
  for (let i = 0; i < position.count; i++) {
    n.copy(sums.get(key(i))!).normalize().toArray(out, i * 3);
  }
  geometry.setAttribute("outlineNormal", new BufferAttribute(out, 3));
}

/**
 * An outline `width` CSS pixels wide around a mesh's visible silhouette. The mesh is drawn
 * again as a hull, each vertex pushed outward on screen along its `outlineNormal`. The hull
 * skips the depth test and draws only where no masking mesh is visible (see `maskOutlines`),
 * which leaves just the part sticking out past the silhouette. A depth-tested hull would
 * keep each vertex's depth, so along an edge where the mesh rests on a surface, the hull
 * would fall behind the nearer surface in front of that edge.
 *
 * The geometry needs `addOutlineNormals`, and the outline mesh needs `OUTLINE_RENDER_ORDER`.
 * The outline also shows through anything that does not mask it.
 */
export function createOutlineMaterial(
  color: ColorRepresentation,
  width: number,
): MeshBasicNodeMaterial {
  const material = new MeshBasicNodeMaterial({ color, side: BackSide });

  // `screenSize` counts device pixels, and the canvas is drawn at the device pixel ratio.
  const widthInPixels = uniform(width * window.devicePixelRatio);
  const outlineNormal = attribute("outlineNormal", "vec3");

  material.vertexNode = Fn(() => {
    const clip = cameraProjectionMatrix
      .mul(modelViewMatrix.mul(vec4(positionLocal, 1)))
      .toVar();
    const clipNormal = cameraProjectionMatrix.mul(
      modelViewMatrix.mul(vec4(outlineNormal, 0)),
    );
    // Which way the vertex moves on screen as it moves along the normal: the derivative of
    // clip.xy / clip.w, scaled from NDC to pixels. Only its direction is used.
    const direction = clipNormal.xy
      .mul(clip.w)
      .sub(clip.xy.mul(clipNormal.w))
      .mul(screenSize)
      .toVar();
    // A normal pointing straight at the camera has no screen direction and stays put.
    const offsetInPixels = direction
      .div(max(direction.length(), 1e-6))
      .mul(widthInPixels);
    const offsetInNdc = offsetInPixels.mul(2).div(screenSize);
    return vec4(clip.xy.add(offsetInNdc.mul(clip.w)), clip.zw);
  })();

  material.depthTest = false;
  material.depthWrite = false;
  material.stencilWrite = true;
  material.stencilFunc = NotEqualStencilFunc;
  material.stencilRef = OUTLINE_MASK;

  return material;
}
