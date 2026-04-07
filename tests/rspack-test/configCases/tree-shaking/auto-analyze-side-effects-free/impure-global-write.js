import { record } from './tracker';

export function impureGlobalWrite() {
  globalThis.__AUTO_SIDE_EFFECTS_WRITE__ =
    (globalThis.__AUTO_SIDE_EFFECTS_WRITE__ || 0) + 1;
  return record('global-write');
}
