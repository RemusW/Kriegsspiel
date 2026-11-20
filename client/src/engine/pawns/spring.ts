/**
 * A damped spring's character. `frequency` is how many times a second it would swing with no
 * damping; `damping` is the damping ratio: 1 settles as fast as possible without overshooting,
 * below 1 overshoots and wobbles, above 1 creeps in slowly.
 */
export interface SpringTune {
  frequency: number;
  damping: number;
}

/** One value on a spring, and how fast it is changing. */
export interface SpringState {
  value: number;
  velocity: number;
}

/** Springs are stepped at this fixed rate, so they behave the same at any frame rate. */
export const SPRING_STEP = 1 / 240;

/**
 * Advance `state` towards `target` by `dt` seconds, which should be at most `SPRING_STEP`.
 * Semi-implicit Euler: the velocity is updated first and the position from the new velocity,
 * which keeps a stiff spring stable at this step size.
 */
export function stepSpring(
  state: SpringState,
  target: number,
  tune: SpringTune,
  dt: number,
) {
  const omega = 2 * Math.PI * tune.frequency;
  const acceleration =
    omega * omega * (target - state.value) -
    2 * tune.damping * omega * state.velocity;
  state.velocity += acceleration * dt;
  state.value += state.velocity * dt;
}

/** A spring at rest at this value. */
export function springAt(value: number): SpringState {
  return { value, velocity: 0 };
}
