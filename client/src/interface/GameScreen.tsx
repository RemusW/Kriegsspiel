import { useGame } from "../useGame";
import styles from "./GameScreen.module.css";
import { Toolbar } from "./Toolbar";

export function GameScreen() {
  const { board, connection } = useGame();

  return (
    <>
      <Toolbar />
      <div className={styles.status}>
        Server: {connection} | Units: {board.units.length}
      </div>
    </>
  );
}
