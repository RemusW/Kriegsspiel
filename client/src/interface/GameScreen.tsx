import { useClientStore } from "../state/store";
import { useGameConnection } from "../useGameConnection";
import styles from "./GameScreen.module.css";
import { Toolbar } from "./Toolbar";

export function GameScreen() {
  useGameConnection();
  const connection = useClientStore((s) => s.connection);
  const pawns = useClientStore((s) => s.board.pawns);

  return (
    <>
      <Toolbar />
      <div className={styles.status}>
        Server: {connection} | Units: {pawns.length}
      </div>
    </>
  );
}
