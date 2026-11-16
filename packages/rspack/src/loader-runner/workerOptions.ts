import { createRequire } from 'node:module';
import type { BroadcastChannel } from 'node:worker_threads';
import type { Compiler } from '../Compiler';

const require = createRequire(import.meta.url);

// Only owned configuration snapshots belong here. Module, compilation and
// loader execution state must remain local to the current task.
class WorkerOptionsCache {
  private entries = new Map<string, { bytes: Buffer; value: unknown }>();

  get<T>(key: string, bytes: Buffer, create: () => T): T {
    const cached = this.entries.get(key);
    if (cached?.bytes.equals(bytes)) return cached.value as T;
    const value = create();
    this.entries.set(key, { bytes, value });
    return value;
  }

  delete(key: string): void {
    this.entries.delete(key);
  }
}

const workerOptions = new Map<number, WorkerOptionsCache>();
let releaseChannel: BroadcastChannel | undefined;

function getReleaseChannel(): BroadcastChannel | undefined {
  if (IS_BROWSER || process.env.WASM) return;
  if (!releaseChannel) {
    const { BroadcastChannel } =
      require('node:worker_threads') as typeof import('node:worker_threads');
    // Different copies of core can have independent compiler handle spaces.
    releaseChannel = new BroadcastChannel(
      `rspack-loader-options:${process.pid}:${import.meta.dirname}`,
    );
    releaseChannel.onmessage = ({ data }: { data: number }) => {
      workerOptions.delete(data);
    };
    releaseChannel.unref();
  }
  return releaseChannel;
}

export function getWorkerOptionsCache(owner: number): WorkerOptionsCache {
  let cache = workerOptions.get(owner);
  if (!cache) {
    getReleaseChannel();
    cache = new WorkerOptionsCache();
    workerOptions.set(owner, cache);
  }
  return cache;
}

export function releaseWorkerOptions(owner: number): void {
  workerOptions.delete(owner);
  getReleaseChannel()?.postMessage(owner);
}

export function reuseWorkerOptions<T>(
  cache: WorkerOptionsCache,
  key: string,
  value: T,
): T {
  if (IS_BROWSER || process.env.WASM) return value;
  const { serialize } = require('node:v8') as typeof import('node:v8');
  let bytes: Buffer;
  try {
    bytes = serialize(value);
  } catch {
    // Structured clone accepts values such as SharedArrayBuffer that V8's
    // serializer cannot encode. Keep supporting them without memoization.
    cache.delete(key);
    return value;
  }
  return cache.get(key, bytes, () => value);
}

interface CompilerWorkerOptions {
  owner: number;
  owners: Set<number>;
}
const compilerWorkers = new WeakMap<Compiler, CompilerWorkerOptions>();
const rootOwners = new WeakMap<Compiler, Set<number>>();
const compilerFinalizer = new FinalizationRegistry<CompilerWorkerOptions>(
  ({ owner, owners }) => {
    if (owners.delete(owner)) releaseWorkerOptions(owner);
  },
);

export function registerCompilerWorkerOptions(
  compiler: Compiler,
  owner: number,
): void {
  if (compilerWorkers.has(compiler)) return;
  let owners = rootOwners.get(compiler.root);
  if (!owners) {
    owners = new Set();
    rootOwners.set(compiler.root, owners);
  }
  owners.add(owner);
  const entry = { owner, owners };
  compilerWorkers.set(compiler, entry);
  // Holdings contain no reference to the compiler or its native instance.
  compilerFinalizer.register(compiler, entry, compiler);
}

export function releaseCompilerWorkerOptions(compiler: Compiler): void {
  const entry = compilerWorkers.get(compiler);
  if (entry) {
    compilerWorkers.delete(compiler);
    compilerFinalizer.unregister(compiler);
    if (entry.owners.delete(entry.owner)) releaseWorkerOptions(entry.owner);
  }
  // Child compilers need not be explicitly closed by their plugins. Once the
  // root is closed, their worker snapshots must not keep accumulating either.
  const owners = rootOwners.get(compiler);
  if (owners) {
    for (const owner of owners) releaseWorkerOptions(owner);
    owners.clear();
    rootOwners.delete(compiler);
  }
}
