import { Canvas } from "@react-three/fiber";
import type { Board, Side, Unit } from "../model";

// World conventions: Y is up, the map lies on the XZ plane at y = 0, north is -Z.
const MAP_WIDTH = 100;
const DEFAULT_MAP_DEPTH = 70;
/** Size of an unscaled unit block: width, thickness, depth. */
const BODY: [number, number, number] = [4, 0.5, 2];

const SIDE_COLOR: Record<Side, string> = {
  Blue: "#8cc8ff",
  Red: "#ff968c",
};

function UnitBlock({ unit }: { unit: Unit }) {
  return (
    <mesh
      position={[unit.pos[0], (BODY[1] * unit.scale) / 2, unit.pos[1]]}
      rotation={[0, unit.rot, 0]}
      scale={unit.scale}
    >
      <boxGeometry args={BODY} />
      <meshBasicMaterial color={SIDE_COLOR[unit.side]} />
    </mesh>
  );
}

export function Table({ board }: { board: Board }) {
  return (
    // Straight down from above, with north at the top of the screen.
    <Canvas camera={{ position: [0, 95, 0], up: [0, 0, -1], fov: 45 }}>
      <color attach="background" args={["#1f2126"]} />
      <mesh rotation={[-Math.PI / 2, 0, 0]}>
        <planeGeometry args={[MAP_WIDTH, DEFAULT_MAP_DEPTH]} />
        <meshBasicMaterial color="#ccc29e" />
      </mesh>
      {board.units.map((unit) => (
        <UnitBlock key={unit.id} unit={unit} />
      ))}
    </Canvas>
  );
}
