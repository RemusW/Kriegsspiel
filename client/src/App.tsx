import { useEffect, useRef } from "react";
import "./App.css";
import { KriegsspielEngine } from "./engine/KriegsspielEngine";
import { useGame } from "./useGame";

export default function App() {
  const { board, connection } = useGame();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const engine = new KriegsspielEngine(canvasRef.current!);
    engine.init().catch(console.error);
    return () => { engine.dispose() };
  }, []);

  return (
    <div className="app">
      <canvas ref={canvasRef} className="editor"/>
      <div className="status">
        Server: {connection} | Units: {board.units.length}
      </div>
    </div>
  );
}
