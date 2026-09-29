import util from 'node:util';
import type { Modules as BindingModules } from '@rspack/binding';
import type { Module } from './Module';

class ModulesWrapper implements ReadonlySet<Module> {
  #inner: BindingModules;

  constructor(inner: BindingModules) {
    this.#inner = inner;
  }

  get size(): number {
    return this.#inner.size();
  }

  has(value: Module): boolean {
    return this.#inner.has(value);
  }

  keys(): SetIterator<Module> {
    return this.values();
  }

  values(): SetIterator<Module> {
    return this.#inner.values().values();
  }

  *entries(): SetIterator<[Module, Module]> {
    for (const value of this.#inner.values()) {
      yield [value, value];
    }
  }

  *[Symbol.iterator](): SetIterator<Module> {
    yield* this.#inner.values();
  }

  forEach(
    callback: (value: Module, key: Module, set: ReadonlySet<Module>) => void,
    thisArg?: unknown,
  ): void {
    if (typeof callback !== 'function') {
      throw new TypeError('callback must be a function');
    }
    for (const value of this.#inner.values()) {
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

// Native composition methods require a Set receiver. Materialize one only
// when a plugin explicitly requests a composition operation.
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
    Object.defineProperty(ModulesWrapper.prototype, name, {
      configurable: true,
      writable: true,
      value(this: ModulesWrapper, other: ReadonlySet<Module>) {
        return method.call(new Set(this.values()), other);
      },
    });
  }
}

export function createModules(inner: BindingModules): ReadonlySet<Module> {
  return new ModulesWrapper(inner);
}
