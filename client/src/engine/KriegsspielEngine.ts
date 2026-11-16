import { BoxGeometry, Mesh, MeshBasicMaterial, PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";


export class KriegsspielEngine {
    private renderer: WebGPURenderer;
    private camera: PerspectiveCamera;
    private scene: Scene;

    private resizeObserver: ResizeObserver;

    constructor(canvas: HTMLCanvasElement) {
        this.renderer = new WebGPURenderer({ canvas: canvas, antialias: true });
        this.renderer.setPixelRatio(window.devicePixelRatio);
        this.renderer.setAnimationLoop(() => this.render());
        this.camera = new PerspectiveCamera(45, 1, 0.1, 1000);
        this.camera.position.set(0, 0, 5);

        this.scene = new Scene();

        this.resizeObserver = new ResizeObserver(() => this.resize());
        this.resizeObserver.observe(canvas);
        this.resize();
        
    }

    async init() {
        await this.renderer.init();

        const geometry = new BoxGeometry(1, 1, 1);
        const material = new MeshBasicMaterial({
            color: 0xffff00
        });
        const box = new Mesh(geometry, material);
        this.scene.add(box);
    }

    dispose() {
        this.renderer.setAnimationLoop(null);
        this.resizeObserver.disconnect();
        this.scene.dispose();
        this.renderer.dispose();
    }

    private resize() {
        const canvas = this.renderer.domElement;
        const width = canvas.clientWidth;
        const height = canvas.clientHeight;
        if (width === 0 || height === 0) return;
        this.renderer.setSize(width, height, false);
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
    }

    render() {
        this.renderer.render(this.scene, this.camera);
    }

}