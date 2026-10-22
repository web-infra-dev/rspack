import util from 'node:util';

const wrappers = new WeakMap<readonly unknown[], ReadonlySet<unknown>>();

/**
 * A read-only view over a shared array of unique values.
 * The owner may update the array in place while preserving Set uniqueness.
 */
export class ReadonlySet<T> implements globalThis.ReadonlySet<T> {
  readonly #values: readonly T[];

  constructor(values: readonly T[]) {
    this.#values = values;
  }

  get size() {
    return this.#values.length;
  }

  has(value: T) {
    return this.#values.includes(value);
  }

  keys(): IterableIterator<T> {
    return this.values();
  }

  values(): IterableIterator<T> {
    return this.#values.values();
  }

  *entries(): IterableIterator<[T, T]> {
    for (const value of this.#values) {
      yield [value, value];
    }
  }

  [Symbol.iterator](): IterableIterator<T> {
    return this.values();
  }

  forEach(
    callback: (value: T, key: T, set: globalThis.ReadonlySet<T>) => void,
    thisArg?: unknown,
  ): void {
    if (typeof callback !== 'function') {
      throw new TypeError('callback must be a function');
    }
    for (const value of this.#values) {
      callback.call(thisArg, value, value, this);
    }
  }

  get [Symbol.toStringTag]() {
    return 'Set';
  }

  [util.inspect.custom]() {
    return new Set(this);
  }
}

// Native composition methods require a real Set receiver. Materialize one only
// for these explicit operations, avoiding a native Set for ordinary reads.
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
    Object.defineProperty(ReadonlySet.prototype, name, {
      configurable: true,
      writable: true,
      value<T>(this: ReadonlySet<T>, other: globalThis.ReadonlySet<T>) {
        return method.call(new Set(this), other);
      },
    });
  }
}

/** Reuse the same view for the same backing array, without retaining it globally. */
export function createReadonlySet<T>(values: readonly T[]): ReadonlySet<T> {
  const cached = wrappers.get(values);
  if (cached) return cached as ReadonlySet<T>;
  const wrapper = new ReadonlySet(values);
  wrappers.set(values, wrapper);
  return wrapper;
}
