import { useEffect, useRef, useState } from "react";
import "./App.css";
import { KSEngine } from "./engine/KriegsspielEngine";
import { GameScreen } from "./interface/GameScreen";
import { Menu } from "./interface/Menu";
import { clientStore } from "./state/store";

type Screen = "menu" | "game";

export default function App() {
  const [screen, setScreen] = useState<Screen>("menu");
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const engine = new KSEngine(canvasRef.current!, clientStore);
    engine.init().catch(console.error);
    return () => {
      engine.dispose();
    };
  }, []);

  // The canvas stays mounted on every screen, so the engine is created once and
  // initialises behind the menu. Unmounting it would run the effect's cleanup and dispose the engine.
  return (
    <div className="app">
      <canvas ref={canvasRef} className="editor" />
      {screen === "menu" ? (
        <Menu
          onHost={() => setScreen("game")}
          onJoin={() => setScreen("game")}
          onSettings={() => {}}
        />
      ) : (
        <GameScreen />
      )}
    </div>
  );
}
