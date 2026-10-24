import util from 'node:util';

export interface ReadonlySetBinding<T> {
  size(): number;
  has(value: T): boolean;
  values(): readonly T[];
}

const wrappers = new WeakMap<
  ReadonlySetBinding<unknown>,
  ReadonlySet<unknown>
>();

/** A read-only facade over a source of unique values. */
class ReadonlySetImpl<T> implements ReadonlySet<T> {
  readonly #binding: ReadonlySetBinding<T>;

  constructor(binding: ReadonlySetBinding<T>) {
    this.#binding = binding;
  }

  get size() {
    return this.#binding.size();
  }

  has(value: T) {
    return this.#binding.has(value);
  }

  keys(): IterableIterator<T> {
    return this.values();
  }

  values(): IterableIterator<T> {
    return this.#binding.values().values();
  }

  *entries(): IterableIterator<[T, T]> {
    for (const value of this.#binding.values()) {
      yield [value, value];
    }
  }

  *[Symbol.iterator](): IterableIterator<T> {
    yield* this.#binding.values();
  }

  forEach(
    callback: (value: T, key: T, set: ReadonlySet<T>) => void,
    thisArg?: unknown,
  ): void {
    if (typeof callback !== 'function') {
      throw new TypeError('callback must be a function');
    }
    for (const value of this.#binding.values()) {
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

export function createReadonlySet<T>(
  binding: ReadonlySetBinding<T>,
): ReadonlySet<T> {
  const cached = wrappers.get(binding);
  if (cached) {
    // A binding object always has the same element type as its wrapper.
    return cached as ReadonlySet<T>;
  }
  const wrapper = new ReadonlySetImpl(binding);
  wrappers.set(binding, wrapper);
  return wrapper;
}
