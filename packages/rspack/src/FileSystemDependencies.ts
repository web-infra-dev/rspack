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

const wrappers = new WeakMap<
  BindingFileSystemDependencies,
  FileSystemDependencies
>();

class FileSystemDependenciesWrapper implements FileSystemDependencies {
  #inner: BindingFileSystemDependencies;
  #pendingAdditions: string[] = [];
  #pendingDeletions: string[] = [];
  #scheduled = false;

  constructor(inner: BindingFileSystemDependencies) {
    this.#inner = inner;
  }

  #scheduleFlush() {
    if (this.#scheduled) return;
    this.#scheduled = true;
    queueMicrotask(() => {
      this.#scheduled = false;
      this.#flush();
    });
  }

  #flush() {
    if (
      this.#pendingAdditions.length === 0 &&
      this.#pendingDeletions.length === 0
    )
      return;
    const additions = this.#pendingAdditions;
    const deletions = this.#pendingDeletions;
    this.#pendingAdditions = [];
    this.#pendingDeletions = [];
    this.#inner.update(additions, deletions);
  }

  #getValues() {
    this.#flush();
    // The binding updates one shared array in place. Iterators and callbacks
    // keep an independent snapshot of the values visible at their start.
    return this.#inner.values().slice();
  }

  get size() {
    this.#flush();
    return this.#inner.size();
  }

  has(value: string) {
    if (typeof value !== 'string') return false;
    this.#flush();
    return this.#inner.has(value);
  }

  add(value: string): this {
    // Additions are applied before deletions in each batch. A later add cancels
    // a queued deletion; Rust clears any deletion from an earlier batch.
    const index = this.#pendingDeletions.indexOf(value);
    if (index !== -1) this.#pendingDeletions.splice(index, 1);
    this.#pendingAdditions.push(value);
    this.#scheduleFlush();
    return this;
  }

  addAll(deps: Iterable<string>): void {
    for (const value of deps) this.add(value);
  }

  delete(value: string): boolean {
    if (typeof value !== 'string' || this.#pendingDeletions.includes(value))
      return false;
    // Compute the synchronous return value without flushing preceding deletes.
    if (!this.#pendingAdditions.includes(value) && !this.#inner.has(value))
      return false;
    this.#pendingDeletions.push(value);
    this.#scheduleFlush();
    return true;
  }

  clear(): void {
    this.#flush();
    this.#inner.clear();
  }

  keys(): IterableIterator<string> {
    return this.values();
  }

  values(): IterableIterator<string> {
    return this.#getValues().values();
  }

  entries(): IterableIterator<[string, string]> {
    return this.#getValues()
      .map((value): [string, string] => [value, value])
      .values();
  }

  *[Symbol.iterator](): IterableIterator<string> {
    // Preserve the existing facade's lazy iterator-start boundary.
    yield* this.#getValues();
  }

  forEach(
    callback: (value: string, key: string, set: FileSystemDependencies) => void,
    thisArg?: unknown,
  ): void {
    for (const value of this.#getValues()) {
      callback.call(thisArg, value, value, this);
    }
  }

  get [Symbol.toStringTag]() {
    return 'Set';
  }

  [util.inspect.custom]() {
    return new Set(this.values());
  }
}

// Native composition methods require a Set receiver. Install shared methods
// only when the runtime supports them, materializing a snapshot on demand.
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
    Object.defineProperty(FileSystemDependenciesWrapper.prototype, name, {
      configurable: true,
      writable: true,
      value(this: FileSystemDependenciesWrapper, other: ReadonlySet<string>) {
        return method.call(new Set(this.values()), other);
      },
    });
  }
}

export function createFileSystemDependencies(
  adm: BindingFileSystemDependencies,
): FileSystemDependencies {
  const cached = wrappers.get(adm);
  if (cached) return cached;
  const wrapper = new FileSystemDependenciesWrapper(adm);
  wrappers.set(adm, wrapper);
  return wrapper;
}
