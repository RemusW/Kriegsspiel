import {
  DoubleSide,
  Mesh,
  MeshBasicNodeMaterial,
  PlaneGeometry,
  Vector2,
} from "three/webgpu";
import {
  fract,
  min,
  mix,
  screenSize,
  select,
  uniform,
  uv,
  vec4,
} from "three/tsl";
import { OVERLAY_LAYER } from "../layers";
import type { ScreenBox } from "./ToolInput";

const COLOR = 0xffd400;
const FILL_OPACITY = 0.1;
const GAP_OPACITY = 0.35;
/** Border width and dash length, in CSS pixels. */
const BORDER_WIDTH = 1;
const DASH_LENGTH = 6;

/**
 * The marquee rectangle: a quad placed directly in clip space, so it ignores the camera and
 * covers exactly the dragged box. Its shader draws a dashed border around a faint fill.
 */
export class MarqueeBox {
  readonly mesh: Mesh;

  /** Corners in normalised device coordinates: bottom left and top right. */
  private min = uniform(new Vector2());
  private max = uniform(new Vector2());

  constructor() {
    const material = new MeshBasicNodeMaterial({
      color: COLOR,
      transparent: true,
      depthTest: false,
      depthWrite: false,
      side: DoubleSide,
    });
    material.vertexNode = vec4(mix(this.min, this.max, uv()), 0, 1);

    // `screenSize` counts device pixels, so CSS lengths are scaled by the pixel ratio.
    const pixelRatio = window.devicePixelRatio;
    const size = this.max.sub(this.min).mul(screenSize).mul(0.5);
    const local = uv().mul(size);
    const edgeDistance = min(
      min(local.x, size.x.sub(local.x)),
      min(local.y, size.y.sub(local.y)),
    );
    // x + y runs along every edge, so one pattern dashes all four sides.
    const dash = fract(local.x.add(local.y).div(DASH_LENGTH * 2 * pixelRatio));
    material.opacityNode = select(
      edgeDistance.lessThan(BORDER_WIDTH * pixelRatio),
      select(dash.lessThan(0.5), 1, GAP_OPACITY),
      FILL_OPACITY,
    );

    this.mesh = new Mesh(new PlaneGeometry(1, 1), material);
    // Its geometry's bounds say nothing about where it is drawn.
    this.mesh.frustumCulled = false;
    this.mesh.layers.set(OVERLAY_LAYER);
    this.mesh.visible = false;
  }

  /** Cover this box, given in the same client coordinates as `canvas`. */
  show(box: ScreenBox, canvas: DOMRect) {
    const ndcX = (x: number) => ((x - canvas.left) / canvas.width) * 2 - 1;
    // Client y grows downward, NDC y upward.
    const ndcY = (y: number) => 1 - ((y - canvas.top) / canvas.height) * 2;
    this.min.value.set(ndcX(box.left), ndcY(box.bottom));
    this.max.value.set(ndcX(box.right), ndcY(box.top));
    this.mesh.visible = true;
  }

  hide() {
    this.mesh.visible = false;
  }

  dispose() {
    this.mesh.geometry.dispose();
    (this.mesh.material as MeshBasicNodeMaterial).dispose();
  }
}
