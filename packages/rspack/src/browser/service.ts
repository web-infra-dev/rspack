import { registerMainThreadJsValueRelease } from '@rspack/binding';

const loaderOptions = new Map<number, object | undefined>();
const loaderAdditionalData = new Map<number, unknown>();
const loaderOptionHandlesByOwner = new WeakMap<
  object,
  WeakMap<object, number>
>();
const loaderOptionOwnerHandles = new WeakMap<object, Set<number>>();
const loaderOptionHandleWithoutOptions = new WeakMap<object, number>();
let nextLoaderOptionsHandle = 1;
let nextLoaderAdditionalDataHandle = 1;
let mainThreadJsValueReleaseRegistered = false;

export function ensureNativeLoaderWorkers(): void {
  // Node worker threads are unavailable in the browser build.
}

export function serializeLoaderOptions(
  value: unknown,
  _owner?: object,
): string {
  return JSON.stringify(value);
}

export function registerLoaderOptions(
  value: object | undefined,
  owner: object,
): number {
  let handles = loaderOptionHandlesByOwner.get(owner);
  if (!handles) {
    handles = new WeakMap();
    loaderOptionHandlesByOwner.set(owner, handles);
  }
  const existing = value
    ? handles.get(value)
    : loaderOptionHandleWithoutOptions.get(owner);
  if (existing !== undefined && loaderOptions.has(existing)) return existing;
  if (nextLoaderOptionsHandle > 0xffffffff) {
    throw new Error('Loader options handle exceeded u32::MAX');
  }
  const handle = nextLoaderOptionsHandle++;
  if (value) handles.set(value, handle);
  else loaderOptionHandleWithoutOptions.set(owner, handle);
  loaderOptions.set(handle, value);
  let ownerHandles = loaderOptionOwnerHandles.get(owner);
  if (!ownerHandles) {
    ownerHandles = new Set();
    loaderOptionOwnerHandles.set(owner, ownerHandles);
  }
  ownerHandles.add(handle);
  return handle;
}

export function getLoaderOptions(handle: number): object | undefined {
  if (!loaderOptions.has(handle)) {
    throw new Error(`Unknown loader options handle ${handle}`);
  }
  return loaderOptions.get(handle);
}

export function getLoaderInputFileSystem(_handle: number): null {
  return null;
}

export function getLoaderCompilerBridge(_handle: number): any {
  throw new Error('Loader workers are unavailable in the browser build');
}

export function clearWorkerLoaderBridgeData(): void {
  // The browser build has no worker-thread bridge cache.
}

export function registerLoaderAdditionalData(value: unknown): number {
  if (!mainThreadJsValueReleaseRegistered) {
    registerMainThreadJsValueRelease(releaseLoaderAdditionalData);
    mainThreadJsValueReleaseRegistered = true;
  }
  if (nextLoaderAdditionalDataHandle > 0xffffffff) {
    throw new Error('Loader additional data handle exceeded u32::MAX');
  }
  const handle = nextLoaderAdditionalDataHandle++;
  loaderAdditionalData.set(handle, value);
  return handle;
}

export function getLoaderAdditionalData(handle: number): unknown {
  if (!loaderAdditionalData.has(handle)) {
    throw new Error(`Unknown loader additional data handle ${handle}`);
  }
  return loaderAdditionalData.get(handle);
}

export function releaseLoaderAdditionalData(handle: number): void {
  loaderAdditionalData.delete(handle);
}

export function releaseLoaderFunctions(owner: object): void {
  const handles = loaderOptionOwnerHandles.get(owner);
  if (!handles) return;
  loaderOptionOwnerHandles.delete(owner);
  loaderOptionHandlesByOwner.delete(owner);
  loaderOptionHandleWithoutOptions.delete(owner);
  for (const handle of handles) loaderOptions.delete(handle);
}

export function deserializeLoaderOptions(value: string): unknown {
  return JSON.parse(value);
}

export function markLoaderFunctionThis(): void {
  // The browser build has no worker-thread function bridge.
}
