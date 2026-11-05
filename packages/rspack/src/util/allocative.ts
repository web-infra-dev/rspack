const snapshotHook = '__rspack_internal_runJsSnapshot';

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
    value(path: string) {
      if (typeof globalThis.gc !== 'function') {
        throw new Error(
          'Allocative heap snapshots require Node.js --expose-gc. Run the profiling process with node --expose-gc.',
        );
      }
      const { writeHeapSnapshot } =
        require('node:v8') as typeof import('node:v8');
      globalThis.gc();
      writeHeapSnapshot(path);
    },
  });
}
