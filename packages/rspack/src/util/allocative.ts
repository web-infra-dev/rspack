import type binding from '@rspack/binding';

const registered = new WeakSet<object>();

/** Register lazily, when the compiler loads its actual native binding. */
export function registerAllocativeSnapshot(
  instanceBinding: typeof binding,
): void {
  if (!process.env.RSPACK_ALLOCATIVE_DIR || registered.has(instanceBinding)) {
    return;
  }
  const enabled = instanceBinding.registerAllocativeSnapshotCallback((path) => {
    if (typeof globalThis.gc !== 'function') {
      throw new Error(
        'Allocative heap snapshots require Node.js --expose-gc. Run the profiling process with node --expose-gc.',
      );
    }
    globalThis.gc();
    const { writeHeapSnapshot } =
      require('node:v8') as typeof import('node:v8');
    writeHeapSnapshot(path);
  });
  if (enabled) registered.add(instanceBinding);
}
