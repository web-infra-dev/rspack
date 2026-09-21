import util from 'node:util';
import type { FileSystemDependencies as BindingFileSystemDependencies } from '@rspack/binding';
import { getArraySnapshot } from './util/arrayHelpers';

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
  #pendingAdditions = new Set<string>();
  #pendingDeletions = new Set<string>();
  #flushScheduled = false;

  constructor(inner: BindingFileSystemDependencies) {
    this.#inner = inner;
  }

  #scheduleFlush() {
    if (this.#flushScheduled) return;
    this.#flushScheduled = true;
    queueMicrotask(() => {
      this.#flushScheduled = false;
      this.#flush();
    });
  }

  #flush() {
    if (this.#pendingAdditions.size === 0 && this.#pendingDeletions.size === 0)
      return;
    const additions = Array.from(this.#pendingAdditions);
    const deletions = Array.from(this.#pendingDeletions);
    this.#pendingAdditions.clear();
    this.#pendingDeletions.clear();
    this.#inner.update(additions, deletions);
  }

  #snapshotValues() {
    this.#flush();
    // Native updates invalidate the cached snapshot. Repeated reads share it;
    // existing iterators and callbacks retain their snapshot across mutations.
    return getArraySnapshot(this.#inner.values());
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
    this.#pendingDeletions.delete(value);
    this.#pendingAdditions.add(value);
    this.#scheduleFlush();
    return this;
  }

  addAll(deps: Iterable<string>): void {
    for (const value of deps) this.add(value);
  }

  delete(value: string): boolean {
    if (typeof value !== 'string' || this.#pendingDeletions.has(value))
      return false;
    // Compute the synchronous return value without flushing preceding deletes.
    if (!this.#pendingAdditions.has(value) && !this.#inner.has(value))
      return false;
    // Keep pending additions in their original order if a later add cancels
    // this deletion before the batch is flushed.
    this.#pendingDeletions.add(value);
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
    return this.#snapshotValues().values();
  }

  entries(): IterableIterator<[string, string]> {
    return this.#snapshotValues()
      .map((value): [string, string] => [value, value])
      .values();
  }

  *[Symbol.iterator](): IterableIterator<string> {
    // Preserve the existing facade's lazy iterator-start boundary.
    yield* this.#snapshotValues();
  }

  forEach(
    callback: (value: string, key: string, set: FileSystemDependencies) => void,
    thisArg?: unknown,
  ): void {
    for (const value of this.#snapshotValues()) {
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
  binding: BindingFileSystemDependencies,
): FileSystemDependencies {
  const cached = wrappers.get(binding);
  if (cached) return cached;
  const wrapper = new FileSystemDependenciesWrapper(binding);
  wrappers.set(binding, wrapper);
  return wrapper;
}
