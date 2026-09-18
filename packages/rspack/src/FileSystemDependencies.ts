import util from 'node:util';
import type { FileSystemDependencies as BindingFileSystemDependencies } from '@rspack/binding';

export interface FileSystemDependencies {
  readonly size: number;
  has(value: string): boolean;
  add(value: string): this;
  addAll(deps: Iterable<string>): void;
  delete(value: string): boolean;
  clear(): void;
  keys(): IterableIterator<string>;
  values(): IterableIterator<string>;
  entries(): IterableIterator<[string, string]>;
  [Symbol.iterator](): IterableIterator<string>;
  forEach(
    callback: (value: string, key: string, set: FileSystemDependencies) => void,
    thisArg?: unknown,
  ): void;
}

const proxies = new WeakMap<
  BindingFileSystemDependencies,
  FileSystemDependencies
>();

export function createFileSystemDependencies(
  adm: BindingFileSystemDependencies,
): FileSystemDependencies {
  const cached = proxies.get(adm);
  if (cached) return cached;

  // Like Diagnostics, the target supplies the JavaScript collection shape;
  // the binding owns the data and implements collection operations.
  const target = new Set<string>();
  const deleted = new Set<string>();
  const pending = new Set<string>();
  const flush = () => {
    const values = Array.from(pending);
    pending.clear();
    adm.addAll(values);
  };
  const add = (value: string) => {
    // Merge additions made in the same turn into one native call. Keep pending
    // values visible to readers until that microtask runs.
    if (pending.size === 0) queueMicrotask(flush);
    pending.add(value);
    deleted.delete(value);
  };
  const getValues = () => {
    const values = adm.values();
    if (pending.size !== 0) {
      const merged = new Set(values);
      for (const value of pending) merged.add(value);
      for (const value of deleted) merged.delete(value);
      return Array.from(merged);
    }
    // The binding updates one shared array in place. Keep iterators and
    // callbacks independent of later reads or additions to that array.
    return deleted.size === 0
      ? values.slice()
      : values.filter((value) => !deleted.has(value));
  };
  const has = (value: string) =>
    typeof value === 'string' &&
    !deleted.has(value) &&
    (pending.has(value) || adm.has(value));
  const values = () => getValues().values();
  const extensions: Record<string | symbol, unknown> = {
    has,
    add(value: string) {
      add(value);
      return proxy;
    },
    addAll(deps: Iterable<string>) {
      for (const value of deps) add(value);
    },
    delete(value: string) {
      if (!has(value)) return false;
      deleted.add(value);
      return true;
    },
    clear() {
      for (const value of getValues()) deleted.add(value);
    },
    keys: values,
    values,
    entries() {
      return getValues()
        .map((value): [string, string] => [value, value])
        .values();
    },
    *[Symbol.iterator]() {
      // Preserve the existing facade's lazy iterator-start boundary.
      yield* getValues();
    },
    forEach(
      callback: (
        value: string,
        key: string,
        set: FileSystemDependencies,
      ) => void,
      thisArg?: unknown,
    ) {
      for (const value of getValues()) {
        callback.call(thisArg, value, value, proxy);
      }
    },
    [util.inspect.custom]() {
      return new Set(getValues());
    },
  };

  Object.defineProperty(target, util.inspect.custom, {
    value: extensions[util.inspect.custom],
  });

  // Native Set methods require a real Set receiver. Follow ObservableSet's
  // approach for the newer composition methods, when the runtime supports them.
  for (const name of [
    'union',
    'intersection',
    'difference',
    'symmetricDifference',
    'isSubsetOf',
    'isSupersetOf',
    'isDisjointFrom',
  ]) {
    const method = Reflect.get(Set.prototype, name);
    if (typeof method === 'function') {
      extensions[name] = (other: ReadonlySet<string>) =>
        method.call(new Set(getValues()), other);
    }
  }

  const proxy = new Proxy(target, {
    get(target, name, receiver) {
      if (name === 'size') {
        return deleted.size !== 0 || pending.size !== 0
          ? getValues().length
          : adm.size();
      }
      if (Object.prototype.hasOwnProperty.call(extensions, name)) {
        return extensions[name];
      }
      return Reflect.get(target, name, receiver);
    },
  }) as unknown as FileSystemDependencies;
  proxies.set(adm, proxy);
  return proxy;
}
