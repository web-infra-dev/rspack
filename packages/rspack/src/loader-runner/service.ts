import { createRequire } from 'node:module';
import path from 'node:path';
import { registerMainThreadJsValueRelease } from '@rspack/binding';

const require = createRequire(import.meta.url);

let nativeWorkers: Set<import('node:worker_threads').Worker> | undefined;
const LOADER_FUNCTION_MARKER = '__rspack_loader_function__';
const LOADER_FUNCTION_THIS_MARKER = '__rspack_loader_function_this__';
const LOADER_LOCAL_REFERENCE_MARKER = '__rspack_loader_local_reference__';
const LOADER_VALUE_TYPE_MARKER = '__rspack_loader_value_type__';
const LOADER_NON_ENUMERABLE_MARKER =
  '__rspack_loader_non_enumerable_properties__';
const loaderFunctions = new Map<number, Function>();
const loaderFunctionOwnerIds = new WeakMap<object, Set<number>>();
const loaderFunctionOwnerFinalizer = new FinalizationRegistry<Set<number>>(
  (functionIds) => {
    for (const functionId of functionIds) loaderFunctions.delete(functionId);
  },
);
const loaderLocalValues = new Map<number, object>();
const loaderLocalValueOwnerIds = new WeakMap<object, Set<number>>();
const loaderLocalValueFinalizer = new FinalizationRegistry<Set<number>>(
  (valueIds) => {
    for (const valueId of valueIds) loaderLocalValues.delete(valueId);
  },
);
const loaderLocalValueIds = new WeakMap<object, LoaderFunctionHandle>();
const loaderFunctionThisSnapshots = new WeakMap<object, object>();
const loaderOptions = new Map<
  number,
  { value: LoaderBridgeData; owner: WeakRef<LoaderOptionsOwner> }
>();
const loaderAdditionalData = new Map<number, unknown>();
const loaderOptionHandlesByOwner = new WeakMap<
  object,
  WeakMap<object, number>
>();
const loaderOptionOwnerHandles = new WeakMap<object, Set<number>>();
const loaderOptionHandleWithoutOptions = new WeakMap<object, number>();
const loaderOptionOwnerFinalizer = new FinalizationRegistry<Set<number>>(
  (handles) => {
    for (const handle of handles) loaderOptions.delete(handle);
  },
);
type LoaderFunctionOwner = 'main' | 'worker';
type LoaderFunctionHandle = { id: number; owner: LoaderFunctionOwner };
type LoaderOptionsOwner = object & {
  context?: string;
  inputFileSystem?: any;
  options?: {
    loader?: Record<string, any>;
    mode?: string;
  };
  __internal__ruleSet?: {
    references: { get(ident: string): any };
  };
  __internal__takeModuleExecutionResult?(id: number): any;
};
type LoaderCompilerBridge = {
  context: string;
  mode?: string;
  loader: Record<string, any>;
  getLoaderOptionsByIdent(ident: string): any;
  takeModuleExecutionResult(id: number): any;
};
type LoaderBridgeData = {
  options?: object;
  inputFileSystem: object | null;
  compiler: LoaderCompilerBridge;
};
const proxyFunctionIds = new WeakMap<Function, LoaderFunctionHandle>();
const loaderInputFileSystemBridges = new WeakMap<object, object>();
const workerLoaderBridgeData = new Map<number, LoaderBridgeData>();
let nextLoaderFunctionId = 1;
let nextLoaderLocalValueId = 1;
let nextLoaderOptionsHandle = 1;
let nextLoaderAdditionalDataHandle = 1;
let mainThreadJsValueReleaseRegistered = false;
let workerFunctionPort: import('node:worker_threads').MessagePort | undefined;
let nextFunctionRequestId = 1;

type SerializedError = {
  name?: string;
  message: string;
  stack?: string;
};

function serializeError(error: unknown): SerializedError {
  if (
    error instanceof Error ||
    (error && typeof error === 'object' && 'message' in error)
  ) {
    return {
      name: (error as Error).name,
      stack: (error as Error).stack,
      message: (error as Error).message,
    };
  }
  return {
    name: 'Error',
    message: typeof error === 'string' ? error : String(error),
  };
}

type FunctionBridgeMessage = {
  type: 'function-invoke';
  requestId: number;
  functionId: number;
  thisArg: any;
  args: any[];
  sharedBuffer: SharedArrayBuffer;
};

type FunctionBridgeResponse = {
  type: 'function-response';
  requestId: number;
  result?: any;
  error?: SerializedError;
};

type LoaderOptionsBridgeMessage = {
  type: 'loader-options-request';
  requestId: number;
  optionsHandle: number;
  sharedBuffer: SharedArrayBuffer;
};

type LoaderOptionsBridgeResponse = {
  type: 'loader-options-response';
  requestId: number;
  result?: any;
  error?: SerializedError;
};

type LoaderAdditionalDataBridgeMessage = {
  type: 'loader-additional-data-request';
  requestId: number;
  action: 'store' | 'get';
  handle?: number;
  value?: unknown;
  sharedBuffer: SharedArrayBuffer;
};

type LoaderAdditionalDataBridgeResponse = {
  type: 'loader-additional-data-response';
  requestId: number;
  handle?: number;
  value?: unknown;
  error?: SerializedError;
};

type LoaderBridgeMessage =
  | FunctionBridgeMessage
  | FunctionBridgeResponse
  | LoaderOptionsBridgeMessage
  | LoaderOptionsBridgeResponse
  | LoaderAdditionalDataBridgeMessage
  | LoaderAdditionalDataBridgeResponse;

function currentFunctionOwner(): LoaderFunctionOwner {
  const { isMainThread } =
    require('node:worker_threads') as typeof import('node:worker_threads');
  return isMainThread ? 'main' : 'worker';
}

function createLoaderInputFileSystemBridge(
  owner: LoaderOptionsOwner,
): object | null {
  if (!owner.inputFileSystem) return null;
  const existing = loaderInputFileSystemBridges.get(owner);
  if (existing) return existing;
  const ownerRef = new WeakRef(owner);
  const getFileSystem = () => {
    const fileSystem = ownerRef.deref()?.inputFileSystem;
    if (!fileSystem) throw new Error('Loader input file system is unavailable');
    return fileSystem;
  };
  const toMetadata = (stats: any) => ({
    isFile: stats.isFile(),
    isDirectory: stats.isDirectory(),
    isSymlink: stats.isSymbolicLink(),
    atimeMs: stats.atimeMs ?? stats.atime?.getTime(),
    mtimeMs: stats.mtimeMs ?? stats.mtime?.getTime(),
    ctimeMs: stats.ctimeMs ?? stats.ctime?.getTime(),
    size: stats.size,
  });
  const bridge = {
    readFile(
      path: string,
      callback: (error: Error | null, value?: any) => void,
    ) {
      const fileSystem = getFileSystem();
      fileSystem.readFile.call(fileSystem, path, callback);
    },
    readFileSync(path: string) {
      const fileSystem = getFileSystem();
      return fileSystem.readFileSync.call(fileSystem, path);
    },
    readdir(
      path: string,
      callback: (error: Error | null, value?: string[]) => void,
    ) {
      const fileSystem = getFileSystem();
      fileSystem.readdir.call(fileSystem, path, callback);
    },
    readdirSync(path: string) {
      const fileSystem = getFileSystem();
      return fileSystem.readdirSync.call(fileSystem, path);
    },
    stat(path: string, callback: (error: Error | null, value?: any) => void) {
      const fileSystem = getFileSystem();
      fileSystem.stat.call(
        fileSystem,
        path,
        (error: Error | null, value?: any) =>
          callback(error, value ? toMetadata(value) : undefined),
      );
    },
    statSync(path: string) {
      const fileSystem = getFileSystem();
      return toMetadata(fileSystem.statSync.call(fileSystem, path));
    },
  };
  loaderInputFileSystemBridges.set(owner, bridge);
  return bridge;
}

function createLoaderCompilerBridge(
  owner: LoaderOptionsOwner,
): LoaderCompilerBridge {
  const ownerRef = new WeakRef(owner);
  const getOwner = () => {
    const owner = ownerRef.deref();
    if (!owner) throw new Error('Loader compiler bridge is unavailable');
    return owner;
  };
  return {
    context: owner.context ?? '',
    mode: owner.options?.mode,
    // main's parallel loader model copies compiler.options.loader onto the worker context.
    loader: { ...(owner.options?.loader ?? {}) },
    getLoaderOptionsByIdent: (ident: string) =>
      getOwner().__internal__ruleSet?.references.get(ident),
    takeModuleExecutionResult: (id: number) =>
      getOwner().__internal__takeModuleExecutionResult?.(id),
  };
}

/** Keeps loader-owned JS values in the main isolate behind one opaque handle. */
export function registerLoaderOptions(
  value: object | undefined,
  owner: LoaderOptionsOwner,
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
  loaderOptions.set(handle, {
    value: {
      options: value,
      inputFileSystem: createLoaderInputFileSystemBridge(owner),
      compiler: createLoaderCompilerBridge(owner),
    },
    owner: new WeakRef(owner),
  });
  let ownerHandles = loaderOptionOwnerHandles.get(owner);
  if (!ownerHandles) {
    ownerHandles = new Set();
    loaderOptionOwnerHandles.set(owner, ownerHandles);
    loaderOptionOwnerFinalizer.register(owner, ownerHandles, owner);
  }
  ownerHandles.add(handle);
  return handle;
}

/** Returns the original main-isolate object or synchronously requests a worker-local clone. */
function getLoaderBridgeData(handle: number): LoaderBridgeData {
  if (currentFunctionOwner() === 'main') {
    const entry = loaderOptions.get(handle);
    if (!entry) throw new Error(`Unknown loader options handle ${handle}`);
    return entry.value;
  }
  const cached = workerLoaderBridgeData.get(handle);
  if (cached) return cached;
  const port = workerFunctionPort;
  if (!port) {
    throw new Error('Loader options bridge is not registered in this worker');
  }
  const value = requestRemoteLoaderOptions(port, handle) as LoaderBridgeData;
  workerLoaderBridgeData.set(handle, value);
  return value;
}

export function getLoaderOptions(handle: number): object | undefined {
  return getLoaderBridgeData(handle).options;
}

export function getLoaderInputFileSystem(handle: number): any {
  return getLoaderBridgeData(handle).inputFileSystem;
}

export function getLoaderCompilerBridge(handle: number): LoaderCompilerBridge {
  return getLoaderBridgeData(handle).compiler;
}

export function clearWorkerLoaderBridgeData(): void {
  workerLoaderBridgeData.clear();
}

function storeMainLoaderAdditionalData(value: unknown): number {
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

/** Stores a loader batch result in the main isolate and returns its native handle. */
export function registerLoaderAdditionalData(value: unknown): number {
  if (currentFunctionOwner() === 'main') {
    return storeMainLoaderAdditionalData(value);
  }
  const port = workerFunctionPort;
  if (!port) {
    throw new Error(
      'Loader additional data bridge is not registered in this worker',
    );
  }
  const response = requestRemoteLoaderAdditionalData(port, {
    action: 'store',
    value: encodeFunctionBridgeValue(value),
  });
  if (response.handle === undefined) {
    throw new Error('Loader additional data bridge returned no handle');
  }
  return response.handle;
}

/** Resolves a native additional-data handle in main or clones it into the current worker. */
export function getLoaderAdditionalData(handle: number): unknown {
  if (currentFunctionOwner() === 'main') {
    if (!loaderAdditionalData.has(handle)) {
      throw new Error(`Unknown loader additional data handle ${handle}`);
    }
    return loaderAdditionalData.get(handle);
  }
  const port = workerFunctionPort;
  if (!port) {
    throw new Error(
      'Loader additional data bridge is not registered in this worker',
    );
  }
  return decodeFunctionBridgeValue(
    requestRemoteLoaderAdditionalData(port, { action: 'get', handle }).value,
    port,
  );
}

/** Called by the native handle's Drop implementation on the main isolate. */
export function releaseLoaderAdditionalData(handle: number): void {
  loaderAdditionalData.delete(handle);
}

export function serializeLoaderOptions(value: unknown, owner?: object): string {
  const v8 = require('node:v8') as typeof import('node:v8');
  let encoded = encodeFunctionBridgeValue(value, owner);
  if (owner && value && typeof value === 'object') {
    let handle = loaderLocalValueIds.get(value);
    if (!handle || !loaderLocalValues.has(handle.id)) {
      handle = {
        id: nextLoaderLocalValueId++,
        owner: currentFunctionOwner(),
      };
      loaderLocalValueIds.set(value, handle);
      loaderLocalValues.set(handle.id, value);
      let valueIds = loaderLocalValueOwnerIds.get(owner);
      if (!valueIds) {
        valueIds = new Set();
        loaderLocalValueOwnerIds.set(owner, valueIds);
        loaderLocalValueFinalizer.register(owner, valueIds, owner);
      }
      valueIds.add(handle.id);
    }
    encoded = {
      [LOADER_LOCAL_REFERENCE_MARKER]: handle,
      value: encoded,
    };
  }
  return v8.serialize(encoded).toString('base64');
}

export function releaseLoaderFunctions(owner: object): void {
  const functionIds = loaderFunctionOwnerIds.get(owner);
  if (functionIds) {
    loaderFunctionOwnerIds.delete(owner);
    loaderFunctionOwnerFinalizer.unregister(owner);
    for (const functionId of functionIds) loaderFunctions.delete(functionId);
  }
  const valueIds = loaderLocalValueOwnerIds.get(owner);
  if (valueIds) {
    loaderLocalValueOwnerIds.delete(owner);
    loaderLocalValueFinalizer.unregister(owner);
    for (const valueId of valueIds) loaderLocalValues.delete(valueId);
  }
  const optionHandles = loaderOptionOwnerHandles.get(owner);
  if (optionHandles) {
    loaderOptionOwnerHandles.delete(owner);
    loaderOptionHandlesByOwner.delete(owner);
    loaderOptionHandleWithoutOptions.delete(owner);
    loaderOptionOwnerFinalizer.unregister(owner);
    for (const handle of optionHandles) loaderOptions.delete(handle);
  }
}

/**
 * Prevents a method call on loaderContext from serializing the native-backed module/compiler
 * facade as its `this` value. Only user-defined context fields participate in the temporary
 * function bridge snapshot; the native accessor phase replaces this snapshot with a task handle.
 */
export function markLoaderFunctionThis(
  target: object,
  customFields: object,
): void {
  loaderFunctionThisSnapshots.set(target, customFields);
}

function encodeFunctionBridgeValue(value: any, owner?: object): any {
  const normalized = new WeakMap<object, any>();
  const normalize = (current: any): any => {
    if (typeof current === 'function') {
      let handle = proxyFunctionIds.get(current);
      if (!handle) {
        handle = { id: nextLoaderFunctionId++, owner: currentFunctionOwner() };
        loaderFunctions.set(handle.id, current);
        if (owner) {
          let functionIds = loaderFunctionOwnerIds.get(owner);
          if (!functionIds) {
            functionIds = new Set();
            loaderFunctionOwnerIds.set(owner, functionIds);
            loaderFunctionOwnerFinalizer.register(owner, functionIds, owner);
          }
          functionIds.add(handle.id);
        }
      }
      return { [LOADER_FUNCTION_MARKER]: handle };
    }
    if (!current || typeof current !== 'object') return current;
    const loaderContextSnapshot = loaderFunctionThisSnapshots.get(current);
    if (loaderContextSnapshot) {
      return {
        [LOADER_FUNCTION_THIS_MARKER]: normalize(loaderContextSnapshot),
      };
    }
    if (Buffer.isBuffer(current)) {
      return {
        [LOADER_VALUE_TYPE_MARKER]: 'Buffer',
        value: current,
      };
    }
    if (
      current instanceof ArrayBuffer ||
      ArrayBuffer.isView(current) ||
      current instanceof Date ||
      current instanceof RegExp ||
      current instanceof Error
    ) {
      return current;
    }
    if (current instanceof URL) {
      return {
        [LOADER_VALUE_TYPE_MARKER]: 'URL',
        value: current.href,
      };
    }
    if (current instanceof URLSearchParams) {
      return {
        [LOADER_VALUE_TYPE_MARKER]: 'URLSearchParams',
        value: current.toString(),
      };
    }
    const existing = normalized.get(current);
    if (existing) return existing;
    const result: any =
      current instanceof Map
        ? new Map()
        : current instanceof Set
          ? new Set()
          : Array.isArray(current)
            ? []
            : {};
    normalized.set(current, result);
    if (current instanceof Map) {
      for (const [key, item] of current) {
        result.set(normalize(key), normalize(item));
      }
    } else if (current instanceof Set) {
      for (const item of current) result.add(normalize(item));
    } else if (Array.isArray(current)) {
      for (const item of current) result.push(normalize(item));
      for (const key of Object.keys(current)) {
        if (!/^(0|[1-9]\d*)$/.test(key)) {
          result[key] = normalize((current as any)[key]);
        }
      }
    } else {
      for (const key of Object.keys(current)) {
        result[key] = normalize(current[key]);
      }
    }
    const nonEnumerableProperties = Object.getOwnPropertyNames(current)
      .filter(
        (key) =>
          key !== 'length' &&
          !Object.prototype.propertyIsEnumerable.call(current, key),
      )
      .map((key) => [key, normalize(current[key])]);
    if (nonEnumerableProperties.length > 0) {
      result[LOADER_NON_ENUMERABLE_MARKER] = nonEnumerableProperties;
    }
    return result;
  };
  return normalize(value);
}

export function deserializeLoaderOptions(serialized: string): any {
  const v8 = require('node:v8') as typeof import('node:v8');
  const encoded = v8.deserialize(Buffer.from(serialized, 'base64'));
  const handle = encoded?.[LOADER_LOCAL_REFERENCE_MARKER] as
    LoaderFunctionHandle | undefined;
  if (handle?.owner === currentFunctionOwner()) {
    const local = loaderLocalValues.get(handle.id);
    if (local) return local;
  }
  return decodeFunctionBridgeValue(handle ? encoded.value : encoded);
}

function decodeFunctionBridgeValue(
  value: any,
  remotePort?: import('node:worker_threads').MessagePort,
): any {
  const revived = new WeakMap<object, any>();
  const revive = (current: any): any => {
    if (!current || typeof current !== 'object') return current;
    if (current[LOADER_VALUE_TYPE_MARKER] === 'URL') {
      return new URL(current.value);
    }
    if (current[LOADER_VALUE_TYPE_MARKER] === 'URLSearchParams') {
      return new URLSearchParams(current.value);
    }
    if (current[LOADER_VALUE_TYPE_MARKER] === 'Buffer') {
      return Buffer.from(current.value);
    }
    if (
      Object.keys(current).length === 1 &&
      typeof current[LOADER_FUNCTION_MARKER]?.id === 'number'
    ) {
      const handle = current[LOADER_FUNCTION_MARKER] as LoaderFunctionHandle;
      const local =
        handle.owner === currentFunctionOwner()
          ? loaderFunctions.get(handle.id)
          : undefined;
      if (local) return local;
      const proxy = function loaderFunctionProxy(this: any, ...args: any[]) {
        const port = workerFunctionPort ?? remotePort;
        if (!port) {
          throw new Error(
            'Loader function bridge is not registered in this worker',
          );
        }
        return invokeRemoteLoaderFunction(port, handle.id, this, args);
      };
      proxyFunctionIds.set(proxy, handle);
      return proxy;
    }
    if (
      Object.keys(current).length === 1 &&
      current[LOADER_FUNCTION_THIS_MARKER]
    ) {
      return revive(current[LOADER_FUNCTION_THIS_MARKER]);
    }
    if (
      current instanceof ArrayBuffer ||
      ArrayBuffer.isView(current) ||
      current instanceof Date ||
      current instanceof RegExp ||
      current instanceof Error
    ) {
      return current;
    }
    const existing = revived.get(current);
    if (existing) return existing;
    revived.set(current, current);
    const reviveProperties = () => {
      for (const key of Object.keys(current)) {
        if (key !== LOADER_NON_ENUMERABLE_MARKER) {
          current[key] = revive(current[key]);
        }
      }
      const nonEnumerableProperties = current[LOADER_NON_ENUMERABLE_MARKER];
      if (nonEnumerableProperties) {
        delete current[LOADER_NON_ENUMERABLE_MARKER];
        for (const [key, item] of nonEnumerableProperties) {
          Object.defineProperty(current, key, {
            configurable: true,
            value: revive(item),
          });
        }
      }
    };
    if (Array.isArray(current)) {
      reviveProperties();
      return current;
    }
    if (current instanceof Map) {
      const entries = [...current];
      current.clear();
      for (const [key, item] of entries) {
        current.set(revive(key), revive(item));
      }
      return current;
    }
    if (current instanceof Set) {
      const items = [...current];
      current.clear();
      for (const item of items) current.add(revive(item));
      return current;
    }
    reviveProperties();
    return current;
  };
  return revive(value);
}

function invokeRemoteLoaderFunction(
  port: import('node:worker_threads').MessagePort,
  functionId: number,
  thisArg: any,
  args: any[],
): any {
  const requestId = nextFunctionRequestId++;
  const sharedBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT);
  const state = new Int32Array(sharedBuffer);
  port.postMessage({
    type: 'function-invoke',
    requestId,
    functionId,
    thisArg: encodeFunctionBridgeValue(thisArg),
    args: encodeFunctionBridgeValue(args),
    sharedBuffer,
  } satisfies FunctionBridgeMessage);
  const { receiveMessageOnPort } =
    require('node:worker_threads') as typeof import('node:worker_threads');
  while (Atomics.load(state, 0) === 0) {
    Atomics.wait(state, 0, 0, 20);
    let incoming = receiveMessageOnPort(port)?.message as
      LoaderBridgeMessage | undefined;
    while (incoming) {
      if (
        incoming.type === 'function-response' &&
        incoming.requestId === requestId
      ) {
        if (incoming.error) {
          const error = new Error(incoming.error.message);
          error.name = incoming.error.name ?? 'Error';
          error.stack = incoming.error.stack;
          throw error;
        }
        return decodeFunctionBridgeValue(incoming.result, port);
      }
      if (incoming.type === 'function-invoke') {
        handleLoaderFunctionCall(port, incoming);
      }
      incoming = receiveMessageOnPort(port)?.message;
    }
  }
  const response = receiveMessageOnPort(port)?.message as
    FunctionBridgeResponse | undefined;
  if (!response || response.requestId !== requestId) {
    throw new Error('Loader function bridge returned an invalid response');
  }
  if (response.error) {
    const error = new Error(response.error.message);
    error.name = response.error.name ?? 'Error';
    error.stack = response.error.stack;
    throw error;
  }
  return decodeFunctionBridgeValue(response.result, port);
}

function requestRemoteLoaderOptions(
  port: import('node:worker_threads').MessagePort,
  optionsHandle: number,
): any {
  const requestId = nextFunctionRequestId++;
  const sharedBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT);
  const state = new Int32Array(sharedBuffer);
  port.postMessage({
    type: 'loader-options-request',
    requestId,
    optionsHandle,
    sharedBuffer,
  } satisfies LoaderOptionsBridgeMessage);
  const { receiveMessageOnPort } =
    require('node:worker_threads') as typeof import('node:worker_threads');
  while (Atomics.load(state, 0) === 0) {
    Atomics.wait(state, 0, 0, 20);
    let incoming = receiveMessageOnPort(port)?.message as
      LoaderBridgeMessage | undefined;
    while (incoming) {
      if (
        incoming.type === 'loader-options-response' &&
        incoming.requestId === requestId
      ) {
        if (incoming.error) throw deserializeError(incoming.error);
        return decodeFunctionBridgeValue(incoming.result, port);
      }
      if (incoming.type === 'function-invoke') {
        handleLoaderFunctionCall(port, incoming);
      }
      incoming = receiveMessageOnPort(port)?.message as
        LoaderBridgeMessage | undefined;
    }
  }
  const response = receiveMessageOnPort(port)?.message as
    LoaderOptionsBridgeResponse | undefined;
  if (!response || response.requestId !== requestId) {
    throw new Error('Loader options bridge returned an invalid response');
  }
  if (response.error) throw deserializeError(response.error);
  return decodeFunctionBridgeValue(response.result, port);
}

function requestRemoteLoaderAdditionalData(
  port: import('node:worker_threads').MessagePort,
  request: Pick<
    LoaderAdditionalDataBridgeMessage,
    'action' | 'handle' | 'value'
  >,
): LoaderAdditionalDataBridgeResponse {
  const requestId = nextFunctionRequestId++;
  const sharedBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT);
  const state = new Int32Array(sharedBuffer);
  port.postMessage({
    type: 'loader-additional-data-request',
    requestId,
    ...request,
    sharedBuffer,
  } satisfies LoaderAdditionalDataBridgeMessage);
  const { receiveMessageOnPort } =
    require('node:worker_threads') as typeof import('node:worker_threads');
  while (Atomics.load(state, 0) === 0) {
    Atomics.wait(state, 0, 0, 20);
    let incoming = receiveMessageOnPort(port)?.message as
      LoaderBridgeMessage | undefined;
    while (incoming) {
      if (
        incoming.type === 'loader-additional-data-response' &&
        incoming.requestId === requestId
      ) {
        if (incoming.error) throw deserializeError(incoming.error);
        return incoming;
      }
      if (incoming.type === 'function-invoke') {
        handleLoaderFunctionCall(port, incoming);
      }
      incoming = receiveMessageOnPort(port)?.message as
        LoaderBridgeMessage | undefined;
    }
  }
  const response = receiveMessageOnPort(port)?.message as
    LoaderAdditionalDataBridgeResponse | undefined;
  if (
    !response ||
    response.type !== 'loader-additional-data-response' ||
    response.requestId !== requestId
  ) {
    throw new Error(
      'Loader additional data bridge returned an invalid response',
    );
  }
  if (response.error) throw deserializeError(response.error);
  return response;
}

function deserializeError(error: SerializedError): Error {
  const result = new Error(error.message);
  result.name = error.name ?? 'Error';
  result.stack = error.stack;
  return result;
}

export function setLoaderFunctionBridge(
  port: import('node:worker_threads').MessagePort,
): void {
  workerFunctionPort = port;
  listenForLoaderFunctionCalls(port);
}

function listenForLoaderFunctionCalls(
  port: import('node:worker_threads').MessagePort,
): void {
  port.on('message', (message: LoaderBridgeMessage) => {
    if (message.type === 'function-invoke') {
      handleLoaderFunctionCall(port, message);
    } else if (message.type === 'loader-options-request') {
      handleLoaderOptionsRequest(port, message);
    } else if (message.type === 'loader-additional-data-request') {
      handleLoaderAdditionalDataRequest(port, message);
    }
  });
}

function handleLoaderAdditionalDataRequest(
  port: import('node:worker_threads').MessagePort,
  message: LoaderAdditionalDataBridgeMessage,
): void {
  const state = new Int32Array(message.sharedBuffer);
  try {
    if (message.action === 'store') {
      const handle = storeMainLoaderAdditionalData(
        decodeFunctionBridgeValue(message.value, port),
      );
      port.postMessage({
        type: 'loader-additional-data-response',
        requestId: message.requestId,
        handle,
      } satisfies LoaderAdditionalDataBridgeResponse);
    } else {
      if (
        message.handle === undefined ||
        !loaderAdditionalData.has(message.handle)
      ) {
        throw new Error(
          `Unknown loader additional data handle ${message.handle}`,
        );
      }
      // The response is structured-cloned into the worker that owns the next parallel batch.
      port.postMessage({
        type: 'loader-additional-data-response',
        requestId: message.requestId,
        value: encodeFunctionBridgeValue(
          loaderAdditionalData.get(message.handle),
        ),
      } satisfies LoaderAdditionalDataBridgeResponse);
    }
  } catch (error) {
    port.postMessage({
      type: 'loader-additional-data-response',
      requestId: message.requestId,
      error: serializeError(error),
    } satisfies LoaderAdditionalDataBridgeResponse);
  } finally {
    Atomics.add(state, 0, 1);
    Atomics.notify(state, 0);
  }
}

function handleLoaderOptionsRequest(
  port: import('node:worker_threads').MessagePort,
  message: LoaderOptionsBridgeMessage,
): void {
  const state = new Int32Array(message.sharedBuffer);
  try {
    const entry = loaderOptions.get(message.optionsHandle);
    if (!entry) {
      throw new Error(`Unknown loader options handle ${message.optionsHandle}`);
    }
    // MessagePort applies the structured clone algorithm to the function-free envelope.
    port.postMessage({
      type: 'loader-options-response',
      requestId: message.requestId,
      result: encodeFunctionBridgeValue(entry.value, entry.owner.deref()),
    } satisfies LoaderOptionsBridgeResponse);
  } catch (error) {
    // A structured-clone failure must still wake the blocked worker with an actionable error.
    port.postMessage({
      type: 'loader-options-response',
      requestId: message.requestId,
      error: serializeError(error),
    } satisfies LoaderOptionsBridgeResponse);
  } finally {
    Atomics.add(state, 0, 1);
    Atomics.notify(state, 0);
  }
}

function handleLoaderFunctionCall(
  port: import('node:worker_threads').MessagePort,
  message: FunctionBridgeMessage,
): void {
  const state = new Int32Array(message.sharedBuffer);
  let response: FunctionBridgeResponse;
  try {
    const fn = loaderFunctions.get(message.functionId);
    if (!fn) throw new Error(`Unknown loader function ${message.functionId}`);
    response = {
      type: 'function-response',
      requestId: message.requestId,
      result: encodeFunctionBridgeValue(
        Reflect.apply(
          fn,
          decodeFunctionBridgeValue(message.thisArg, port),
          decodeFunctionBridgeValue(message.args, port),
        ),
      ),
    };
  } catch (error) {
    response = {
      type: 'function-response',
      requestId: message.requestId,
      error: serializeError(error),
    };
  }
  port.postMessage(response);
  Atomics.add(state, 0, 1);
  Atomics.notify(state, 0);
}

/** Starts process-wide workers which directly receive tasks from the native MPMC queue. */
export function ensureNativeLoaderWorkers(workerOptions?: {
  maxWorkers?: number;
}): void {
  if (nativeWorkers) return;

  const { MessageChannel, Worker } =
    require('node:worker_threads') as typeof import('node:worker_threads');
  const cpus = require('node:os').cpus().length;
  const availableThreads = Math.max(cpus - 1, 1);
  const configuredWorkers = workerOptions?.maxWorkers
    ? Math.max(Math.floor(workerOptions.maxWorkers), 1)
    : undefined;
  const rawWorkersFromEnv = Number.parseInt(
    process.env.RSPACK_LOADER_WORKER_THREADS || '',
    10,
  );
  const workersFromEnv =
    Number.isFinite(rawWorkersFromEnv) && rawWorkersFromEnv > 0
      ? Math.floor(rawWorkersFromEnv)
      : undefined;
  const count = configuredWorkers || workersFromEnv || availableThreads;
  nativeWorkers = new Set();

  const spawnWorker = (slot: number, restartCount: number): void => {
    const { port1: mainFunctionPort, port2: workerFunctionPort } =
      new MessageChannel();
    listenForLoaderFunctionCalls(mainFunctionPort);
    mainFunctionPort.unref();
    const worker = new Worker(path.resolve(import.meta.dirname, 'worker.js'), {
      workerData: { rspackNativeLoaderWorker: true, workerFunctionPort },
      transferList: [workerFunctionPort],
    });
    nativeWorkers!.add(worker);
    let workerReady = false;
    worker.on('message', (message) => {
      if (message?.type !== 'rspack-loader-worker-ready') return;
      workerReady = true;
    });
    // A worker error is followed by exit. Installing the listener prevents Node from turning a
    // recoverable worker-slot failure into an uncaught exception in the compiler's main isolate.
    worker.once('error', () => {});
    worker.once('exit', () => {
      nativeWorkers?.delete(worker);
      mainFunctionPort.close();
      const nextRestartCount = workerReady ? 0 : Math.min(restartCount + 1, 7);
      const restartDelay = Math.min(10 * 2 ** nextRestartCount, 1000);
      const timer = setTimeout(
        () => spawnWorker(slot, nextRestartCount),
        restartDelay,
      );
      timer.unref();
    });
    // Workers are persistent and shared by every compiler in this process, but they should not
    // keep an otherwise idle process alive.
    worker.unref();
  };

  for (let index = 0; index < count; index++) spawnWorker(index, 0);
}
