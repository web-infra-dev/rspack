import {
  allocativeBindingMarker,
  allocativeRootMarker,
  parseHeapSnapshot,
  redactHeapSnapshot,
  stringifyRedactedHeapSnapshot,
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

function writeRedactedSnapshot(
  path: string,
  snapshot: ReturnType<typeof redactHeapSnapshot>['redacted'],
): void {
  const { randomUUID } = require('node:crypto') as typeof import('node:crypto');
  const { closeSync, openSync, renameSync, rmSync, writeFileSync } =
    require('node:fs') as typeof import('node:fs');
  const temporary = `${path}.${randomUUID()}.tmp`;
  const fd = openSync(temporary, 'wx', 0o600);
  try {
    try {
      for (const chunk of stringifyRedactedHeapSnapshot(snapshot)) {
        writeFileSync(fd, chunk);
      }
    } finally {
      closeSync(fd);
    }
    renameSync(temporary, path);
  } catch (error) {
    rmSync(temporary, { force: true });
    throw error;
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
      const { getHeapSnapshot, writeHeapSnapshot } =
        require('node:v8') as typeof import('node:v8');
      const { mkdirSync } = require('node:fs') as typeof import('node:fs');
      const { dirname } = require('node:path') as typeof import('node:path');
      mkdirSync(dirname(path), { recursive: true, mode: 0o700 });

      if (process.env.RSPACK_ALLOCATIVE_INCLUDE_CONTENT === '1') {
        globalThis.gc();
        writeHeapSnapshot(path, { exposeNumericValues: true });
        return;
      }

      globalThis.gc();
      // V8 serializes the captured graph synchronously when this stream is read.
      // Keep capture, redaction and native traversal in one JS-thread callback.
      const stream = getHeapSnapshot();
      let heap: ReturnType<typeof parseHeapSnapshot>;
      try {
        heap = parseHeapSnapshot(() => stream.read(64 * 1024));
      } finally {
        stream.destroy();
      }
      const { redacted, markers } = redactHeapSnapshot(heap);
      writeRedactedSnapshot(path, redacted);
      return markers;
    },
  });
}
