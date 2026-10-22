import util from 'node:util';

/**
 * A read-only view over an array whose values are already unique.
 * Callers must not mutate the backing array.
 */
export class ReadonlySetImpl<T> implements ReadonlySet<T> {
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
    callback: (value: T, key: T, set: ReadonlySet<T>) => void,
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
    Object.defineProperty(ReadonlySetImpl.prototype, name, {
      configurable: true,
      writable: true,
      value<T>(this: ReadonlySetImpl<T>, other: ReadonlySet<T>) {
        return method.call(new Set(this), other);
      },
    });
  }
}
