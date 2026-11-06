import {
  allocativeRootMarker,
  redactHeapSnapshot,
  type RspackRootKind,
} from './heapSnapshot.mjs';

const snapshotHook = '__rspack_internal_runJsSnapshot';

export function markAllocativeRoot(value: object, kind: RspackRootKind): void {
  if (process.env.RSPACK_ALLOCATIVE_DIR !== undefined) {
    Object.defineProperty(value, allocativeRootMarker, { value: kind });
  }
}

/** Install before creating the first native wrapper for this compiler. */
export function installAllocativeSnapshotHook(): void {
  if (
    process.env.RSPACK_ALLOCATIVE_DIR === undefined ||
    Object.hasOwn(globalThis, snapshotHook)
  ) {
    return;
  }
  Object.defineProperty(globalThis, snapshotHook, {
    configurable: true,
    async value(path: string) {
      if (typeof globalThis.gc !== 'function') {
        throw new Error(
          'Allocative heap snapshots require Node.js --expose-gc. Run the profiling process with node --expose-gc.',
        );
      }
      const { getHeapSnapshot, writeHeapSnapshot } =
        require('node:v8') as typeof import('node:v8');
      const { mkdirSync, writeFileSync } =
        require('node:fs') as typeof import('node:fs');
      const { dirname } = require('node:path') as typeof import('node:path');
      mkdirSync(dirname(path), { recursive: true, mode: 0o700 });

      if (process.env.RSPACK_ALLOCATIVE_INCLUDE_CONTENT === '1') {
        globalThis.gc();
        writeHeapSnapshot(path, { exposeNumericValues: true });
        return;
      }

      globalThis.gc();
      const chunks: string[] = [];
      for await (const chunk of getHeapSnapshot().setEncoding('utf8')) {
        chunks.push(chunk);
      }
      const heap = JSON.parse(chunks.join('')) as Parameters<
        typeof redactHeapSnapshot
      >[0];
      const { redacted, markers } = redactHeapSnapshot(heap);
      writeFileSync(path, JSON.stringify(redacted), { mode: 0o600 });
      return markers;
    },
  });
}
