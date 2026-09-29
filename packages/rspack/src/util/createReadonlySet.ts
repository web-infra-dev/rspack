import util from 'node:util';
import { memoize } from './memoize';

class ReadonlySetWrapper<T> implements ReadonlySet<T> {
  #getSet: () => Set<T>;

  constructor(getValues: () => Iterable<T>) {
    this.#getSet = memoize(() => new Set(getValues()));
  }

  get size(): number {
    return this.#getSet().size;
  }

  has(value: T): boolean {
    return this.#getSet().has(value);
  }

  keys(): SetIterator<T> {
    return this.#getSet().keys();
  }

  values(): SetIterator<T> {
    return this.#getSet().values();
  }

  entries(): SetIterator<[T, T]> {
    return this.#getSet().entries();
  }

  [Symbol.iterator](): SetIterator<T> {
    return this.values();
  }

  forEach(
    callback: (value: T, key: T, set: ReadonlySet<T>) => void,
    thisArg?: unknown,
  ): void {
    if (typeof callback !== 'function') {
      throw new TypeError('callback must be a function');
    }
    this.#getSet().forEach((value) => {
      callback.call(thisArg, value, value, this);
    });
  }

  get [Symbol.toStringTag]() {
    return 'Set';
  }

  [util.inspect.custom]() {
    return this.#getSet();
  }

  static {
    // Native composition methods require a Set receiver.
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
        Object.defineProperty(ReadonlySetWrapper.prototype, name, {
          configurable: true,
          writable: true,
          value(
            this: ReadonlySetWrapper<unknown>,
            other: ReadonlySet<unknown>,
          ) {
            return method.call(this.#getSet(), other);
          },
        });
      }
    }
  }
}

export function createReadonlySet<T>(
  getValues: () => Iterable<T>,
): ReadonlySet<T> {
  return new ReadonlySetWrapper(getValues);
}
