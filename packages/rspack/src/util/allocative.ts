import {
  allocativeBindingMarker,
  allocativeRootMarker,
  type RspackRootKind,
} from './heapSnapshot.mjs';

const snapshotHook = '__rspack_internal_runJsSnapshot';
let snapshotEnabled: boolean | undefined;

function isSnapshotEnabled(): boolean {
  return (snapshotEnabled ??= process.env.RSPACK_ALLOCATIVE_DIR !== undefined);
}

export function markAllocativeRoot(value: object, kind: RspackRootKind): void {
  if (isSnapshotEnabled()) {
    Object.defineProperty(value, allocativeRootMarker, { value: kind });
  }
}

/** The public object already owns this binding; expose that edge to the profiler. */
export function linkAllocativeBinding(value: object, binding: object): void {
  if (isSnapshotEnabled()) {
    Object.defineProperty(value, allocativeBindingMarker, { value: binding });
  }
}

/** Install before creating the first native wrapper for this compiler. */
export function installAllocativeSnapshotHook(): void {
  if (!isSnapshotEnabled() || Object.hasOwn(globalThis, snapshotHook)) {
    return;
  }
  Object.defineProperty(globalThis, snapshotHook, {
    configurable: true,
    value(path: string) {
      if (typeof globalThis.gc !== 'function') {
        throw new Error(
          'Allocative heap snapshots require Node.js --expose-gc. Run the profiling process with node --expose-gc.',
        );
      }
      const { writeHeapSnapshot } =
        require('node:v8') as typeof import('node:v8');
      const { randomUUID } =
        require('node:crypto') as typeof import('node:crypto');
      const { mkdirSync, renameSync, rmSync } =
        require('node:fs') as typeof import('node:fs');
      const { dirname } = require('node:path') as typeof import('node:path');
      mkdirSync(dirname(path), { recursive: true, mode: 0o700 });

      const temporary = `${path}.${randomUUID()}.tmp`;
      globalThis.gc();
      try {
        writeHeapSnapshot(temporary, { exposeNumericValues: true });
        renameSync(temporary, path);
      } catch (error) {
        rmSync(temporary, { force: true });
        throw error;
      }
    },
  });
}
