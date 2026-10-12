import { JsSourceMap } from '@rspack/binding';
import { isNil, serializeObject, toObject } from '../util';

type LoaderSourceMap = string | object | null | undefined;
type BindingSourceMap = JsSourceMap | Uint8Array;
type SourceMapInput = BindingSourceMap | (() => Uint8Array);

interface SourceMapState {
  input: SourceMapInput;
  modified: boolean;
}

const states = new WeakMap<object, SourceMapState>();

function getInput(state: SourceMapState): BindingSourceMap {
  if (typeof state.input === 'function') state.input = state.input();
  return state.input;
}

function createSourceMap(input: SourceMapInput): object {
  const state: SourceMapState = { input, modified: false };
  const target: Record<PropertyKey, unknown> = {};
  let initialized = false;

  const initialize = () => {
    if (initialized) return;
    const input = getInput(state);
    const map = toObject(input instanceof JsSourceMap ? input.toJson() : input);
    for (const key of Reflect.ownKeys(map)) {
      const descriptor = Object.getOwnPropertyDescriptor(map, key)!;
      descriptor.value = track(descriptor.value);
      Object.defineProperty(target, key, descriptor);
    }
    initialized = true;
  };

  // Store tracked nested values in the target itself so reads and descriptors
  // keep satisfying Proxy invariants after a loader freezes or seals the map.
  const track = (value: unknown): unknown => {
    if (!value || typeof value !== 'object') return value;
    for (const key of Reflect.ownKeys(value)) {
      const descriptor = Object.getOwnPropertyDescriptor(value, key)!;
      if ('value' in descriptor) {
        descriptor.value = track(descriptor.value);
        Object.defineProperty(value, key, descriptor);
      }
    }
    return new Proxy(value, handler());
  };

  const handler = (prepare: () => void = () => {}): ProxyHandler<object> => ({
    get(target, key, receiver) {
      prepare();
      return Reflect.get(target, key, receiver);
    },
    has(target, key) {
      prepare();
      return Reflect.has(target, key);
    },
    ownKeys(target) {
      prepare();
      return Reflect.ownKeys(target);
    },
    getOwnPropertyDescriptor(target, key) {
      prepare();
      return Reflect.getOwnPropertyDescriptor(target, key);
    },
    defineProperty(target, key, descriptor) {
      prepare();
      const previous = Reflect.getOwnPropertyDescriptor(target, key);
      const changed =
        !previous ||
        Reflect.ownKeys(descriptor).some(
          (field) =>
            !(field in previous) ||
            !Object.is(
              previous[field as keyof PropertyDescriptor],
              descriptor[field as keyof PropertyDescriptor],
            ),
        );
      const result = Reflect.defineProperty(target, key, descriptor);
      if (result && changed) state.modified = true;
      return result;
    },
    set(target, key, value, receiver) {
      prepare();
      // Reflect.set reaches defineProperty for ordinary writable properties.
      return Reflect.set(target, key, value, receiver);
    },
    deleteProperty(target, key) {
      prepare();
      const existed = Object.hasOwn(target, key);
      const result = Reflect.deleteProperty(target, key);
      if (result && existed) state.modified = true;
      return result;
    },
    setPrototypeOf(target, prototype) {
      prepare();
      const changed = Reflect.getPrototypeOf(target) !== prototype;
      const result = Reflect.setPrototypeOf(target, prototype);
      if (result && changed) state.modified = true;
      return result;
    },
    preventExtensions(target) {
      prepare();
      return Reflect.preventExtensions(target);
    },
  });

  const proxy = new Proxy(target, handler(initialize));
  states.set(proxy, state);
  return proxy;
}

export class SourceMapAdapter {
  static fromBinding(map?: SourceMapInput): object | undefined {
    return isNil(map) ? undefined : createSourceMap(map);
  }

  static toBinding(map: LoaderSourceMap): JsSourceMap | undefined {
    if (isNil(map)) return undefined;
    const state = typeof map === 'object' ? states.get(map) : undefined;
    if (state && !state.modified) {
      const input = getInput(state);
      return input instanceof JsSourceMap
        ? input
        : new JsSourceMap(Buffer.from(input).toString('utf8'));
    }
    const json = typeof map === 'string' ? map : JSON.stringify(map);
    return json ? new JsSourceMap(json) : undefined;
  }

  // Cache entries and worker messages require bytes rather than native classes or Proxies.
  static toSerialized(map: LoaderSourceMap): Uint8Array | undefined {
    if (isNil(map)) return undefined;
    const state = typeof map === 'object' ? states.get(map) : undefined;
    if (state && !state.modified) {
      const input = getInput(state);
      return input instanceof JsSourceMap ? Buffer.from(input.toJson()) : input;
    }
    return serializeObject(map);
  }

  static isUnmodified(map: LoaderSourceMap): boolean {
    if (isNil(map)) return true;
    return typeof map === 'object' && states.get(map)?.modified === false;
  }
}
