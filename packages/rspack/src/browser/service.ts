import { registerMainObjectRelease } from '@rspack/binding';
import type { Compiler } from '../Compiler';

const values = new Map<number, unknown>();
let nextHandle = 1;
let initialized = false;

export function registerMainObject(value: unknown): number {
  if (!initialized) {
    registerMainObjectRelease((handle) => {
      values.delete(handle);
    });
    initialized = true;
  }
  if (nextHandle > 0xffffffff)
    throw new Error('Main object handle exceeded u32::MAX');
  const handle = nextHandle++;
  values.set(handle, value);
  return handle;
}

export function getMainObject(handle: number): unknown {
  if (!values.has(handle))
    throw new Error(`Main object handle ${handle} has been released`);
  return values.get(handle);
}

export function getCompilerHandle(_compiler: Compiler): number {
  return 0;
}
export function registerParallelLoader(..._args: unknown[]): void {}
export function registerLoaderContext(..._args: unknown[]): never {
  throw new Error('Parallel loader workers are unavailable in the browser');
}

export function beginWorkerImport(): () => void {
  return () => {};
}
