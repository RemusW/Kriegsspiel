import type { ReactNode } from "react";
import { useClientStore } from "../state/store";
import styles from "./Toolbar.module.css";

export function Toolbar() {
  const tool = useClientStore((s) => s.tool);
  const marqueeShape = useClientStore((s) => s.marqueeShape);
  const setTool = useClientStore((s) => s.setTool);
  const setMarqueeShape = useClientStore((s) => s.setMarqueeShape);
  const cameraMode = useClientStore((s) => s.cameraMode);
  const setCameraMode = useClientStore((s) => s.setCameraMode);

  return (
    <div
      className={styles.toolbar}
      role="toolbar"
      aria-label="Tools"
      aria-orientation="vertical"
    >
      <ToolButton
        label="Select: click a unit, Shift-click to add or remove one"
        active={tool === "select"}
        onClick={() => setTool("select")}
      >
        <SelectIcon />
      </ToolButton>
      <ToolButton
        label="Marquee: drag a box to select units, Shift to add them"
        active={tool === "marquee"}
        onClick={() => setTool("marquee")}
      >
        {marqueeShape === "box" ? <BoxIcon /> : <LassoIcon />}
      </ToolButton>
      {tool === "marquee" && (
        <div className={styles.options} role="group" aria-label="Marquee shape">
          <ToolButton
            small
            label="Box"
            active={marqueeShape === "box"}
            onClick={() => setMarqueeShape("box")}
          >
            <BoxIcon />
          </ToolButton>
          <ToolButton
            small
            label="Lasso"
            active={marqueeShape === "lasso"}
            onClick={() => setMarqueeShape("lasso")}
          >
            <LassoIcon />
          </ToolButton>
        </div>
      )}
      <ToolButton
        label="Spawn: click the map to place an infantry unit"
        active={tool === "spawn"}
        onClick={() => setTool("spawn")}
      >
        <SpawnIcon />
      </ToolButton>
      <div className={styles.divider} role="separator" />
      <div className={styles.group} role="group" aria-label="Camera">
        <ToolButton
          label="Fly camera: W/A/S/D to move, R/F up and down, right-drag to look"
          active={cameraMode === "fly"}
          onClick={() => setCameraMode("fly")}
        >
          <FlyIcon />
        </ToolButton>
        <ToolButton
          label="Map camera: right-drag or W/A/S/D to pan, middle-drag to tilt, scroll to zoom"
          active={cameraMode === "map"}
          onClick={() => setCameraMode("map")}
        >
          <MapIcon />
        </ToolButton>
      </div>
    </div>
  );
}

interface ToolButtonProps {
  label: string;
  active: boolean;
  small?: boolean;
  onClick: () => void;
  children: ReactNode;
}

function ToolButton({
  label,
  active,
  small,
  onClick,
  children,
}: ToolButtonProps) {
  return (
    <button
      type="button"
      className={small ? `${styles.button} ${styles.small}` : styles.button}
      aria-label={label}
      aria-pressed={active}
      title={label}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

function Icon({ children }: { children: ReactNode }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="20"
      height="20"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

function SelectIcon() {
  return (
    <Icon>
      <path d="M6 3v16l4.5-4.5 3 6.5 2.5-1-3-6.5H19z" />
    </Icon>
  );
}

function BoxIcon() {
  return (
    <Icon>
      <rect x="4" y="4" width="16" height="16" strokeDasharray="3 5" />
    </Icon>
  );
}

function SpawnIcon() {
  return (
    <Icon>
      <rect x="2" y="10" width="14" height="9" rx="1" />
      <path d="M19 3v6M16 6h6" />
    </Icon>
  );
}

function FlyIcon() {
  return (
    <Icon>
      <path d="M21 3 3 10.5l7 2.5 2.5 7z" />
      <path d="m10 13 4-4" />
    </Icon>
  );
}

function MapIcon() {
  return (
    <Icon>
      <path d="M3 6.5 9 4l6 2.5L21 4v13.5L15 20l-6-2.5L3 20z" />
      <path d="M9 4v13.5M15 6.5V20" />
    </Icon>
  );
}

function LassoIcon() {
  return (
    <Icon>
      <ellipse cx="12" cy="10" rx="8" ry="6" strokeDasharray="3 4" />
      <path d="M7.5 15c-1.5 1.5-1.5 4 1 5" />
    </Icon>
  );
}
