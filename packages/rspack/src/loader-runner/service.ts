import { createRequire } from 'node:module';
import path from 'node:path';
import {
  prepareLoaderTask,
  completeLoaderTaskHooks,
  registerLoaderReference,
  getSerializedLoaderOptions,
  registerLoaderWorker,
  registerMainObjectRelease,
  releaseLoaderReferences,
  stopLoaderWorker,
} from '@rspack/binding';
import type { BuildInfo, Dependency, Module } from '@rspack/binding';
import type { Tinypool } from 'tinypool';
import type { Compiler } from '../Compiler';

const require = createRequire(import.meta.url);
type Port = import('node:worker_threads').MessagePort;
type Reference = {
  id: number;
  owner: number;
  kind: 'object' | 'function' | 'array';
};
type WireValue = { ref: Reference } | { value: unknown };
type Request = {
  type: 'request';
  id: number;
  operation:
    | 'get'
    | 'set'
    | 'keys'
    | 'has'
    | 'call'
    | 'main'
    | 'store'
    | 'retain'
    | 'release'
    | 'prepare'
    | 'delete'
    | 'descriptor'
    | 'define';
  owner?: number;
  target: number;
  key?: string;
  args?: WireValue[];
  state?: SharedArrayBuffer;
};
type Response = {
  type: 'response';
  id: number;
  result?: WireValue;
  error?: { message: string; name: string; stack?: string };
};

type Control = { type: 'releaseCompiler'; handle: number };

const values = new Map<number, unknown>();
const referenceFinalizer = new FinalizationRegistry<number>((id) =>
  values.delete(id),
);
const retainedReferences = new Map<number, { value: unknown; count: number }>();
const foreignReferences = new Map<number, Reference[]>();
const workerBridges = new Map<number, ObjectBridge>();
const taskHandles = new Set<number>();
const valueIds = new WeakMap<object, number>();
const remoteReferences = new WeakMap<object, Reference>();
const remoteRoots = new WeakSet<object>();
const compilerHandles = new WeakMap<Compiler, number>();
const compilerIds = new Set<number>();
const referenceVersions = new WeakMap<Compiler, number>();
const parallelCompilers = new WeakSet<Compiler>();

const workerOptions = new Map<
  number,
  { version: number; values: Map<string, unknown> }
>();

function serializeLoaderValue(value: unknown): Buffer | undefined {
  if (IS_BROWSER || process.env.WASM) return undefined;
  const { serialize } = require('node:v8') as typeof import('node:v8');
  try {
    return serialize(value);
  } catch {
    // Leave functions and other non-serializable values on the existing bridge.
    return undefined;
  }
}

function deserializeLoaderValue(bytes: Buffer): unknown {
  const { deserialize } = require('node:v8') as typeof import('node:v8');
  return deserialize(bytes);
}

export function serializeAdditionalData(value: unknown): Buffer | number {
  // V8 turns class instances into plain objects and drops prototype methods.
  // Keep those roots on the existing bridge, outside this plain-data experiment.
  if (
    value !== null &&
    typeof value === 'object' &&
    Object.prototype.toString.call(value) === '[object Object]'
  ) {
    const prototype = Object.getPrototypeOf(value);
    if (prototype !== null && prototype !== Object.prototype)
      return registerMainObject(value);
  }
  return serializeLoaderValue(value) ?? registerMainObject(value);
}

export function deserializeAdditionalData(value: Buffer | number): unknown {
  return typeof value === 'number'
    ? getMainObject(value)
    : deserializeLoaderValue(value);
}

export function releaseWorkerCompiler(handle: number): void {
  workerOptions.delete(handle);
}

export interface WorkerLoaderContext {
  compilerHandle: number;
  referenceVersion: number;
  compiler: Compiler;
  buildInfo: BuildInfo;
  getDependency(id: number): Dependency;
  extensions: Record<string, unknown>;
}

export function createWorkerCompiler(main: WorkerLoaderContext): Compiler {
  let cached = workerOptions.get(main.compilerHandle);
  if (!cached || cached.version !== main.referenceVersion) {
    cached = { version: main.referenceVersion, values: new Map() };
    workerOptions.set(main.compilerHandle, cached);
  }
  const references = main.compiler.__internal__ruleSet.references;
  const get = references.get;
  const cache = cached.values;
  references.get = (ident: string) => {
    if (!cache.has(ident)) {
      const optionsIdent = ident.endsWith('$$parallelism')
        ? ident.slice(0, -'$$parallelism'.length)
        : ident;
      const bytes = getSerializedLoaderOptions(
        main.compilerHandle,
        optionsIdent,
      );
      if (bytes) {
        const value = deserializeLoaderValue(bytes) as {
          options: unknown;
          parallel: boolean | 'strict' | { maxWorkers?: number } | undefined;
        };
        cache.set(optionsIdent, value.options);
        cache.set(`${optionsIdent}$$parallelism`, value.parallel);
      } else {
        cache.set(ident, get(ident));
      }
    }
    return cache.get(ident);
  };
  return main.compiler;
}
const compilerFinalizer = new FinalizationRegistry<number>((handle) => {
  releaseMainObject(handle);
  releaseLoaderReferences(handle);
});
let nextHandle = 1;
let owner = 0;
let bridge: ObjectBridge | undefined;
let registeredRelease = false;

function store(value: unknown): number {
  if (nextHandle > 0xffffffff)
    throw new Error('Main object handle exceeded u32::MAX');
  const handle = nextHandle++;
  values.set(handle, value);
  return handle;
}

function lookup(handle: number): unknown {
  if (!values.has(handle))
    throw new Error(`Main object handle ${handle} has been released`);
  const value = values.get(handle);
  if (value instanceof WeakRef) {
    const target = value.deref();
    if (!target) throw new Error('Loader compiler has been garbage collected');
    return target;
  }
  return value;
}

export function releaseMainObject(handle: number): void {
  values.delete(handle);
  if (compilerIds.delete(handle))
    for (const channel of workerBridges.values())
      channel.releaseCompiler(handle);
  if (taskHandles.delete(handle))
    for (const channel of workerBridges.values()) channel.releaseTask(handle);
  const references = foreignReferences.get(handle);
  foreignReferences.delete(handle);
  if (references)
    for (const ref of references)
      workerBridges.get(ref.owner)?.releaseReference(ref.id);
}

export function registerMainObject(value: unknown): number {
  if (bridge)
    return bridge.sync({
      operation: 'store',
      target: 0,
      args: [bridge.encode(value)],
    });
  if (!registeredRelease) {
    registerMainObjectRelease(releaseMainObject);
    registeredRelease = true;
  }
  return store(value);
}

export function getMainObject<T = unknown>(handle: number): T {
  return (bridge ? bridge.main(handle) : lookup(handle)) as T;
}

export function getCompilerHandle(compiler: Compiler): number {
  let handle = compilerHandles.get(compiler);
  if (handle !== undefined) return handle;
  handle = registerMainObject(new WeakRef(compiler));
  compilerHandles.set(compiler, handle);
  compilerIds.add(handle);
  compilerFinalizer.register(compiler, handle, compiler);
  const id = handle;
  const compilerRef = new WeakRef(compiler);
  compiler.hooks.shutdown.tap('rspack:loader-objects', () => {
    releaseMainObject(id);
    releaseLoaderReferences(id);
  });
  compiler.hooks.make.tapPromise(
    { name: 'rspack:loader-workers', stage: -Infinity },
    async () => {
      // Dynamic Rule.use functions may enable workers later during make.
      compiling = true;
      const compiler = compilerRef.deref();
      if (compiler && parallelCompilers.has(compiler)) {
        await startNativeLoaderWorkers();
      }
    },
  );
  return handle;
}

export function registerParallelLoader(
  compiler: Compiler,
  ident: string,
  parallel: boolean | 'strict' | { maxWorkers?: number } | undefined,
): void {
  referenceVersions.set(compiler, (referenceVersions.get(compiler) ?? 0) + 1);
  const strict = parallel === 'strict';
  const serialized = serializeLoaderValue({
    options: compiler.__internal__ruleSet.references.get(ident),
    parallel,
  });
  const compilerOptions = strict
    ? serializeLoaderValue({
        context: compiler.context,
        options: {
          mode: compiler.options.mode,
          devtool: compiler.options.devtool,
          loader: compiler.options.loader,
        },
        hashFunction: compiler.options.output.hashFunction,
      })
    : undefined;
  if (
    strict &&
    (!serialized || !compilerOptions || IS_BROWSER || process.env.WASM)
  ) {
    throw new Error(
      "parallel: 'strict' requires native workers and serializable loader options",
    );
  }
  const useWorker = registerLoaderReference(
    getCompilerHandle(compiler),
    ident,
    !!parallel,
    serialized,
    strict,
    compilerOptions,
  );
  if (useWorker && !IS_BROWSER && !process.env.WASM) {
    parallelCompilers.add(compiler);
    ensureNativeLoaderWorkers(
      typeof parallel === 'object' ? parallel : undefined,
    );
  }
}

/** A task-scoped bridge keeps native-backed objects on the compiler's JS thread. */
export function registerLoaderContext(
  compiler: Compiler,
  module: Module,
  extensions: object,
): number {
  const compilation = compiler._lastCompilation!;
  const buildInfo = module.buildInfo;
  remoteRoots.add(compilation);
  remoteRoots.add(buildInfo);
  remoteRoots.add(compiler.inputFileSystem!);
  const compilerRef = new WeakRef(compiler);
  const handle = registerMainObject({
    compilerHandle: getCompilerHandle(compiler),
    referenceVersion: referenceVersions.get(compiler) ?? 0,
    compiler: {
      context: compiler.context,
      options: {
        mode: compiler.options.mode,
        devtool: compiler.options.devtool,
        loader: compiler.options.loader,
        cache: false,
      },
      root: {},
      inputFileSystem: compiler.inputFileSystem,
      _lastCompilation: compilation,
      __internal__ruleSet: {
        references: {
          get: (ident: string) =>
            compilerRef.deref()!.__internal__ruleSet.references.get(ident),
        },
      },
      __internal__takeModuleExecutionResult: (id: number) =>
        compilerRef.deref()!.__internal__takeModuleExecutionResult(id),
    },
    buildInfo,
    getDependency(id: number): Dependency {
      // Resolve by identity: main's graph view can filter removed dependencies.
      const find = (
        block: Pick<Module, 'dependencies' | 'blocks'>,
      ): Dependency | undefined => {
        const dependency = block.dependencies.find(
          (dep) => dep.__internal__id === id,
        );
        if (dependency) return dependency;
        for (const child of block.blocks) {
          const dependency = find(child);
          if (dependency) return dependency;
        }
      };
      const dependency = find(module);
      if (!dependency)
        throw new Error(`Dependency ${id} is unavailable in the module graph`);
      return dependency;
    },
    extensions,
  });
  taskHandles.add(handle);
  return handle;
}

/** Synchronous calls use Atomics while pumping reentrant messages on the same port. */
class ObjectBridge {
  private nextRequest = 1;
  private responses = new Map<number, Response>();
  private pending = new Map<
    number,
    { resolve(value: unknown): void; reject(error: Error): void }
  >();
  private proxies = new Map<string, object>();
  private mainObjects = new Map<number, unknown>();
  private depth = 0;
  private closed = false;
  private handles = new Map<number, object>();
  private activeTasks = new Set<number>();

  constructor(private port: Port) {
    port.on('message', (message: Request | Response | Control) =>
      this.receive(message),
    );
    port.on('close', () => {
      this.closed = true;
      for (const pending of this.pending.values())
        pending.reject(new Error('Loader object bridge closed'));
      this.pending.clear();
      this.clear();
    });
  }

  close(): void {
    this.port.close();
  }

  releaseCompiler(handle: number): void {
    this.port.postMessage({ type: 'releaseCompiler', handle });
  }

  clear(): void {
    this.handles.clear();
    this.proxies.clear();
    this.mainObjects.clear();
  }

  releaseTask(handle: number): void {
    if (this.activeTasks.delete(handle) && this.activeTasks.size === 0)
      this.clear();
  }

  releaseReference(id: number): void {
    void this.async({ operation: 'release', target: id }).catch(() => {});
  }

  main(handle: number): unknown {
    if (this.mainObjects.has(handle)) return this.mainObjects.get(handle);
    const value = this.sync({ operation: 'main', target: handle });
    this.mainObjects.set(handle, value);
    return value;
  }

  encode(value: unknown, reference = false): WireValue {
    const seen = new WeakMap<object, unknown>();
    const visit = (item: unknown, force = false): unknown => {
      if (
        (typeof item !== 'object' || item === null) &&
        typeof item !== 'function'
      )
        return item;
      if (Buffer.isBuffer(item)) return { __rspackBuffer: item };
      if (ArrayBuffer.isView(item) || item instanceof ArrayBuffer) return item;
      if (force && owner === 0) remoteRoots.add(item);
      const remote = remoteReferences.get(item);
      if (remote) return { __rspackReference: remote };
      if (
        force ||
        remoteRoots.has(item) ||
        typeof item === 'function' ||
        (!Array.isArray(item) &&
          Object.getPrototypeOf(item) !== Object.prototype &&
          Object.getPrototypeOf(item) !== null &&
          !(item instanceof Map) &&
          !(item instanceof Set) &&
          !(item instanceof Date) &&
          !(item instanceof RegExp) &&
          !(item instanceof Error) &&
          !ArrayBuffer.isView(item) &&
          !(item instanceof ArrayBuffer))
      ) {
        let id = valueIds.get(item);
        if (id === undefined || !values.has(id)) {
          id = store(new WeakRef(item));
          valueIds.set(item, id);
          referenceFinalizer.register(item, id);
        }
        this.handles.set(id, item);
        return {
          __rspackReference: {
            id,
            owner,
            kind:
              typeof item === 'function'
                ? 'function'
                : Array.isArray(item)
                  ? 'array'
                  : 'object',
          },
        };
      }
      if (
        item instanceof Date ||
        item instanceof RegExp ||
        item instanceof Error
      )
        return item;
      if (seen.has(item)) return seen.get(item);
      if (item instanceof Map) {
        const result = new Map<unknown, unknown>();
        seen.set(item, result);
        for (const [key, value] of item) result.set(visit(key), visit(value));
        return result;
      }
      if (item instanceof Set) {
        const result = new Set<unknown>();
        seen.set(item, result);
        for (const value of item) result.add(visit(value));
        return result;
      }
      const result = (Array.isArray(item) ? [] : {}) as Record<string, unknown>;
      seen.set(item, result);
      for (const key of Object.keys(item))
        result[key] = visit((item as Record<string, unknown>)[key]);
      return result;
    };
    return { value: visit(value, reference) };
  }

  private decode(wire?: WireValue): unknown {
    if (!wire) return undefined;
    if ('ref' in wire) return this.proxy(wire.ref);
    const seen = new WeakMap<object, unknown>();
    const visit = (item: unknown): unknown => {
      if (!item || typeof item !== 'object') return item;
      if ('__rspackReference' in item) {
        const ref = item.__rspackReference as Reference;
        return ref.owner === owner ? lookup(ref.id) : this.proxy(ref);
      }
      if ('__rspackBuffer' in item)
        return Buffer.from(item.__rspackBuffer as Uint8Array);
      if (
        ArrayBuffer.isView(item) ||
        item instanceof ArrayBuffer ||
        item instanceof Date ||
        item instanceof RegExp ||
        item instanceof Error
      )
        return item;
      if (seen.has(item)) return seen.get(item);
      seen.set(item, item);
      if (item instanceof Map) {
        const entries = [...item];
        item.clear();
        for (const [key, value] of entries) item.set(visit(key), visit(value));
      } else if (item instanceof Set) {
        const entries = [...item];
        item.clear();
        for (const value of entries) item.add(visit(value));
      } else {
        const record = item as Record<string, unknown>;
        for (const key of Object.keys(record)) record[key] = visit(record[key]);
      }
      return item;
    };
    return visit(wire.value);
  }

  private proxy(ref: Reference): object {
    const key = `${ref.owner}:${ref.id}`;
    const cached = this.proxies.get(key);
    if (cached) return cached;
    const target =
      ref.kind === 'function'
        ? (..._args: unknown[]) => {}
        : ref.kind === 'array'
          ? []
          : {};
    const methods = new Map<string, unknown>();
    const proxy = new Proxy(target, {
      get: (_target, key) => {
        if (key === Symbol.toStringTag) return 'LoaderRemoteObject';
        if (key === Symbol.iterator && ref.kind === 'array')
          return Array.prototype[Symbol.iterator];
        if (typeof key !== 'string') return undefined;
        if (methods.has(key)) return methods.get(key);
        const value = this.sync({
          operation: 'get',
          target: ref.id,
          owner: ref.owner,
          key,
        });
        if (typeof value === 'function') methods.set(key, value);
        return value;
      },
      set: (_target, key, value) => {
        this.sync({
          operation: 'set',
          target: ref.id,
          owner: ref.owner,
          key: String(key),
          args: [this.encode(value)],
        });
        methods.delete(String(key));
        return true;
      },
      has: (_target, key) =>
        this.sync<boolean>({
          operation: 'has',
          target: ref.id,
          owner: ref.owner,
          key: String(key),
        }),
      ownKeys: () =>
        this.sync<string[]>({
          operation: 'keys',
          target: ref.id,
          owner: ref.owner,
        }),
      getOwnPropertyDescriptor: (_target, key) => {
        if (key === 'length' && Array.isArray(target))
          return Reflect.getOwnPropertyDescriptor(target, key);
        const descriptor = this.sync<PropertyDescriptor | undefined>({
          operation: 'descriptor',
          target: ref.id,
          owner: ref.owner,
          key: String(key),
        });
        return descriptor ? { ...descriptor, configurable: true } : undefined;
      },
      deleteProperty: (_target, key) => {
        methods.delete(String(key));
        return this.sync<boolean>({
          operation: 'delete',
          target: ref.id,
          owner: ref.owner,
          key: String(key),
        });
      },
      defineProperty: (_target, key, descriptor) =>
        this.sync<boolean>({
          operation: 'define',
          target: ref.id,
          owner: ref.owner,
          key: String(key),
          args: [this.encode(descriptor)],
        }),
      apply: (_target, thisArg, args) => {
        const request = {
          operation: 'call' as const,
          target: ref.id,
          owner: ref.owner,
          args: [
            this.encode(thisArg),
            ...args.map((arg) => this.encode(arg, owner === 0)),
          ],
        };
        // A callback fired later by main must leave its event loop free. During
        // a synchronous call both sides can instead pump nested requests.
        return owner === 0 && this.depth === 0
          ? this.async(request)
          : this.sync(request);
      },
    });
    remoteReferences.set(proxy, ref);
    this.proxies.set(key, proxy);
    return proxy;
  }

  private result(response: Response): unknown {
    if (response.error) {
      const error = new Error(response.error.message);
      error.name = response.error.name;
      error.stack = response.error.stack;
      throw error;
    }
    return this.decode(response.result);
  }

  sync<T = unknown>(request: Omit<Request, 'id' | 'type'>): T {
    if (this.closed) throw new Error('Loader object bridge closed');
    const id = this.nextRequest++;
    const state = new Int32Array(new SharedArrayBuffer(4));
    this.port.postMessage({
      ...request,
      id,
      type: 'request',
      state: state.buffer,
    });
    const { receiveMessageOnPort } =
      require('node:worker_threads') as typeof import('node:worker_threads');
    while (!this.responses.has(id)) {
      let incoming = receiveMessageOnPort(this.port);
      while (incoming) {
        this.receive(incoming.message);
        incoming = receiveMessageOnPort(this.port);
      }
      if (!this.responses.has(id)) Atomics.wait(state, 0, 0, 10);
    }
    const response = this.responses.get(id)!;
    this.responses.delete(id);
    return this.result(response) as T;
  }

  async(request: Omit<Request, 'id' | 'type'>): Promise<unknown> {
    return new Promise((resolve, reject) => {
      const id = this.nextRequest++;
      this.pending.set(id, { resolve, reject });
      this.port.postMessage({ ...request, id, type: 'request' });
    });
  }

  private receive(message: Request | Response | Control): void {
    if (message.type === 'releaseCompiler') {
      releaseWorkerCompiler(message.handle);
      return;
    }
    if (message.type === 'response') {
      const pending = this.pending.get(message.id);
      if (!pending) {
        this.responses.set(message.id, message);
        return;
      }
      this.pending.delete(message.id);
      try {
        pending.resolve(this.result(message));
      } catch (error) {
        pending.reject(error as Error);
      }
      return;
    }
    if (message.operation === 'prepare') {
      const taskId = this.decode(message.args![0]) as number;
      void Promise.resolve()
        .then(() => {
          // Parallel loaders skip NormalModule's loader hook. Only prepare the
          // main-owned API bridge; the worker constructs its loader context.
          const context = prepareLoaderTask(taskId);
          const handle = registerLoaderContext(
            lookup(message.target) as Compiler,
            context._module!,
            {},
          );
          context.bridgeHandle = handle;
          completeLoaderTaskHooks(taskId, context);
          this.activeTasks.add(handle);
          this.port.postMessage({
            type: 'response',
            id: message.id,
            result: this.encode(lookup(handle)),
          });
        })
        .catch((error: Error) => {
          this.port.postMessage({
            type: 'response',
            id: message.id,
            error: {
              name: error.name,
              message: error.message,
              stack: error.stack,
            },
          });
        });
      return;
    }
    this.depth++;
    let response: Response;
    try {
      const args = message.args?.map((value) => this.decode(value)) ?? [];
      const target =
        message.operation === 'store' ||
        (message.owner !== undefined && message.owner !== owner)
          ? undefined
          : lookup(message.target);
      let result: unknown;
      let reference = false;
      if (message.owner !== undefined && message.owner !== owner) {
        const destination = workerBridges.get(message.owner);
        if (!destination)
          throw new Error('The worker owning this loader object has exited');
        result = destination.sync(message);
      } else
        switch (message.operation) {
          case 'get':
            result = Reflect.get(target as object, message.key!, target);
            reference =
              typeof result === 'object' &&
              result !== null &&
              !ArrayBuffer.isView(result);
            break;
          case 'set':
            result = Reflect.set(
              target as object,
              message.key!,
              args[0],
              target,
            );
            break;
          case 'has':
            result = Reflect.has(target as object, message.key!);
            break;
          case 'delete':
            result = Reflect.deleteProperty(target as object, message.key!);
            break;
          case 'descriptor':
            result = Object.getOwnPropertyDescriptor(target, message.key!);
            break;
          case 'define':
            result = Reflect.defineProperty(
              target as object,
              message.key!,
              args[0] as PropertyDescriptor,
            );
            break;
          case 'keys':
            result = Object.getOwnPropertyNames(target);
            break;
          case 'call':
            result = Reflect.apply(
              target as (...args: unknown[]) => unknown,
              args[0],
              args.slice(1),
            );
            break;
          case 'main':
            result = target;
            if (taskHandles.has(message.target))
              this.activeTasks.add(message.target);
            else
              reference =
                result !== null &&
                (typeof result === 'object' || typeof result === 'function');
            break;
          case 'store': {
            const handle = store(args[0]);
            const refs = new Map<string, Reference>();
            const seen = new WeakSet<object>();
            const collect = (value: unknown): void => {
              if (
                !value ||
                (typeof value !== 'object' && typeof value !== 'function') ||
                seen.has(value) ||
                ArrayBuffer.isView(value) ||
                value instanceof ArrayBuffer
              )
                return;
              seen.add(value);
              const ref =
                remoteReferences.get(value) ??
                ('__rspackReference' in value
                  ? (value.__rspackReference as Reference)
                  : undefined);
              if (ref && ref.owner !== 0)
                refs.set(`${ref.owner}:${ref.id}`, ref);
              else if (value instanceof Map)
                for (const [key, item] of value) {
                  collect(key);
                  collect(item);
                }
              else if (value instanceof Set)
                for (const item of value) collect(item);
              else for (const item of Object.values(value)) collect(item);
            };
            collect(message.args);
            collect(args[0]);
            for (const ref of refs.values())
              workerBridges
                .get(ref.owner)!
                .sync({ operation: 'retain', target: ref.id });
            foreignReferences.set(handle, [...refs.values()]);
            result = handle;
            break;
          }
          case 'retain': {
            const retained = retainedReferences.get(message.target);
            if (retained) retained.count++;
            else
              retainedReferences.set(message.target, {
                value: target,
                count: 1,
              });
            break;
          }
          case 'release': {
            const retained = retainedReferences.get(message.target);
            if (retained && --retained.count === 0)
              retainedReferences.delete(message.target);
            break;
          }
        }
      response = {
        type: 'response',
        id: message.id,
        result: this.encode(result, reference),
      };
      this.port.postMessage(response);
    } catch (error) {
      const e = error instanceof Error ? error : new Error(String(error));
      response = {
        type: 'response',
        id: message.id,
        error: { name: e.name, message: e.message, stack: e.stack },
      };
      this.port.postMessage(response);
    } finally {
      this.depth--;
      if (message.state) {
        const state = new Int32Array(message.state);
        Atomics.store(state, 0, 1);
        Atomics.notify(state, 0);
      }
    }
  }
}

export function setLoaderObjectBridge(port: Port, workerId: number): void {
  owner = workerId;
  bridge = new ObjectBridge(port);
}

export async function prepareWorkerTask(
  mainHandle: number,
  taskId: number,
): Promise<WorkerLoaderContext> {
  return bridge!.async({
    operation: 'prepare',
    target: mainHandle,
    args: [bridge!.encode(taskId)],
  }) as Promise<WorkerLoaderContext>;
}

export function clearWorkerLoaderContext(): void {
  bridge?.clear();
}

let pool: Tinypool | undefined;
let initialization: Promise<void> | undefined;
let compiling = false;
let workerPoolFailed = false;
let pendingInitializations = 0;
let nextSubscriptionRequest = 1;

const loaderWorkers = new Map<number, { subscribe(): Promise<void> }>();

function initializeLoaderWorker(pool: Tinypool) {
  const { MessageChannel } =
    require('node:worker_threads') as typeof import('node:worker_threads');
  const { port1, port2 } = new MessageChannel();
  const { port1: control, port2: workerControl } = new MessageChannel();
  let resolveReady: () => void;
  let rejectReady: (error: Error) => void;
  let reused = false;
  const ready = new Promise<void>((resolve, reject) => {
    resolveReady = resolve;
    rejectReady = reject;
  });
  const pending = new Map<
    number,
    { resolve(): void; reject(error: Error): void }
  >();
  let failure: Error | undefined;
  control.on(
    'message',
    (message: { threadId?: number; id?: number; error?: string }) => {
      if (message.threadId !== undefined) {
        const id = message.threadId;
        reused = loaderWorkers.has(id);
        if (reused) {
          port1.close();
        } else {
          const worker = pool.threads.find((worker) => worker.threadId === id)!;
          registerLoaderWorker(id);
          workerBridges.set(id, new ObjectBridge(port1));
          port1.unref();
          loaderWorkers.set(id, {
            async subscribe() {
              if (failure) throw failure;
              const id = nextSubscriptionRequest++;
              await new Promise<void>((resolve, reject) => {
                pending.set(id, { resolve, reject });
                control.postMessage({ type: 'subscribe', id });
              });
            },
          });
          worker.once('exit', (code) => {
            failure = new Error(`Loader worker exited (${code})`);
            for (const request of pending.values()) request.reject(failure);
            pending.clear();
            loaderWorkers.delete(id);
            // Tinypool replaces the thread. Reserve a receiver until its startup
            // handshake completes, before retiring the dead worker's native state.
            if (!workerPoolFailed) {
              // process.exit() does not emit Node's error event. Notify Tinypool
              // if it has not already removed this thread after an error.
              if (pool.threads.includes(worker)) worker.emit('error', failure);
              queueWorkerInitialization();
            }
            stopLoaderWorker(id);
            workerBridges.get(id)?.close();
            workerBridges.delete(id);
            control.close();
          });
        }
        resolveReady();
        return;
      }
      const request = pending.get(message.id!);
      if (!request) return;
      pending.delete(message.id!);
      if (message.error) request.reject(new Error(message.error));
      else request.resolve();
    },
  );
  const completed = pool
    .run(
      { port: port2, control: workerControl },
      { transferList: [port2, workerControl] },
    )
    .catch((error: Error) => {
      rejectReady(error);
      throw error;
    })
    .finally(() => {
      if (reused) control.close();
      else control.unref();
    });
  void completed.catch(() => {});
  return {
    ready,
    completed,
    release() {
      control.postMessage({ type: 'initialized' });
    },
    close() {
      control.close();
      port1.close();
    },
  };
}

async function initializeLoaderWorkers(): Promise<void> {
  const workers = Array.from({ length: pool!.options.minThreads }, () =>
    initializeLoaderWorker(pool!),
  );
  try {
    // Hold every startup task until all threads have checked in. Otherwise
    // Tinypool could initialize the same idle thread more than once.
    await Promise.all(workers.map((worker) => worker.ready));
    for (const worker of workers) worker.release();
    await Promise.all(workers.map((worker) => worker.completed));
  } catch (error) {
    for (const worker of workers) worker.close();
    throw error;
  }
}

function queueWorkerInitialization(): void {
  registerLoaderWorker(0);
  pendingInitializations++;
  initialization = Promise.resolve(initialization)
    .then(async () => {
      await initializeLoaderWorkers();
      if (compiling) await subscribeLoaderWorkers();
    })
    .catch(async (error) => {
      workerPoolFailed = true;
      await pool?.destroy();
      throw error;
    })
    .finally(() => {
      if (--pendingInitializations === 0) stopLoaderWorker(0);
    });
  void initialization.catch(() => {});
}

async function subscribeLoaderWorkers(): Promise<void> {
  await Promise.all(
    [...loaderWorkers.values()].map((worker) => worker.subscribe()),
  );
}

async function startNativeLoaderWorkers(): Promise<void> {
  compiling = true;
  await initialization;
  await subscribeLoaderWorkers();
}

export function ensureNativeLoaderWorkers(options?: {
  maxWorkers?: number;
}): void {
  if (initialization) return;
  initialization = import('tinypool').then(({ Tinypool }) => {
    const configured =
      options?.maxWorkers ?? Number(process.env.RSPACK_LOADER_WORKER_THREADS);
    const count =
      Number.isFinite(configured) && configured > 0
        ? Math.max(1, Math.floor(configured))
        : Math.max(1, require('node:os').cpus().length - 1);
    pool = new Tinypool({
      filename: path.resolve(import.meta.dirname, 'worker.js'),
      minThreads: count,
      maxThreads: count,
      concurrentTasksPerWorker: 1,
      useAtomics: false,
    });
    // Idle-thread failures are reported here; exit performs native cleanup and
    // starts a handshake for the replacement thread managed by Tinypool.
    pool.on('error', () => {});
  });
  queueWorkerInitialization();
}

let beginImport: (() => () => void) | undefined;
export function setWorkerImportScheduler(start: () => () => void): void {
  beginImport = start;
}
export function beginWorkerImport(): () => void {
  return beginImport?.() ?? (() => {});
}

export function createStrictWorkerCompiler(handle: number): Compiler {
  const bytes = getSerializedLoaderOptions(handle);
  if (!bytes) throw new Error('Strict compiler options are unavailable');
  const compiler = deserializeLoaderValue(bytes) as Compiler & {
    hashFunction: string;
  };
  Object.assign(compiler, {
    root: compiler,
    __internal__ruleSet: {
      references: {
        get(ident: string) {
          const suffix = '$$parallelism';
          const parallel = ident.endsWith(suffix);
          const key = parallel ? ident.slice(0, -suffix.length) : ident;
          const bytes = getSerializedLoaderOptions(handle, key);
          if (!bytes)
            throw new Error(
              "Non-serializable options are unavailable with parallel: 'strict'",
            );
          const value = deserializeLoaderValue(bytes) as {
            options: unknown;
            parallel: unknown;
          };
          return parallel ? value.parallel : value.options;
        },
      },
    },
  });
  return compiler;
}
