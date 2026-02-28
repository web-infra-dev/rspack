import { Chunks } from '@rspack/binding';
import type { Chunk } from './Chunk';

Object.defineProperty(Chunks.prototype, 'entries', {
  enumerable: true,
  configurable: true,
  value(this: Chunks): SetIterator<[Chunk, Chunk]> {
    const chunks = this._values();
    let index = 0;
    return {
      [Symbol.iterator]() {
        return this;
      },
      next(): IteratorResult<[Chunk, Chunk]> {
        if (index < chunks.length) {
          const chunk = chunks[index++];
          return { value: [chunk, chunk], done: false };
        }
        return { value: undefined, done: true };
      },
    };
  },
});

Object.defineProperty(Chunks.prototype, 'values', {
  enumerable: true,
  configurable: true,
  value(this: Chunks): SetIterator<Chunk> {
    return this._values().values();
  },
});

Object.defineProperty(Chunks.prototype, Symbol.iterator, {
  enumerable: true,
  configurable: true,
  value(this: Chunks): SetIterator<Chunk> {
    return this.values();
  },
});

Object.defineProperty(Chunks.prototype, 'keys', {
  enumerable: true,
  configurable: true,
  value(this: Chunks): SetIterator<Chunk> {
    return this.values();
  },
});

Object.defineProperty(Chunks.prototype, 'forEach', {
  enumerable: true,
  configurable: true,
  value(
    this: Chunks,
    callbackfn: (value: Chunk, value2: Chunk, set: ReadonlySet<Chunk>) => void,
    thisArg?: any,
  ): void {
    for (const chunk of this._values()) {
      callbackfn.call(thisArg, chunk, chunk, this);
    }
  },
});

Object.defineProperty(Chunks.prototype, 'has', {
  enumerable: true,
  configurable: true,
  value(this: Chunks, value: Chunk): boolean {
    return this._has(value);
  },
});

declare module '@rspack/binding' {
  interface Chunks {
    [Symbol.iterator](): SetIterator<Chunk>;
    entries(): SetIterator<[Chunk, Chunk]>;
    values(): SetIterator<Chunk>;
    keys(): SetIterator<Chunk>;
    forEach(
      callbackfn: (
        value: Chunk,
        value2: Chunk,
        set: ReadonlySet<Chunk>,
      ) => void,
      thisArg?: any,
    ): void;
    has(value: Chunk): boolean;
  }
}

export default Chunks;
