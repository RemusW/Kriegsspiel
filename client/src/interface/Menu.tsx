import styles from "./Menu.module.css";

interface MenuProps {
  onHost: () => void;
  onJoin: () => void;
  onSettings: () => void;
}

export function Menu({ onHost, onJoin, onSettings }: MenuProps) {
  return (
    <div className={styles.backdrop}>
      <div className={styles.menu}>
        <h1 className={styles.title}>Kriegsspiel</h1>
        <button type="button" className={styles.button} onClick={onHost}>
          Host Game
        </button>
        <button type="button" className={styles.button} onClick={onJoin}>
          Join Game
        </button>
        <button type="button" className={styles.button} onClick={onSettings}>
          Settings
        </button>
      </div>
    </div>
  );
}
