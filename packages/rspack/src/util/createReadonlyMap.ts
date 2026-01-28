export function createReadonlyMap<T>(
  obj: Pick<ReadonlyMap<string, T>, 'get' | 'keys'>,
) {
  return {
    ...obj,
    *values() {
      const keys = this.keys();
      for (const key of keys) {
        yield this.get(key);
      }
    },
    *entries() {
      const keys = this.keys();
      for (const key of keys) {
        yield [key, this.get(key)];
      }
    },
    forEach(
      callback: (
        value: T,
        key: string,
        map: ReadonlyMap<string, Readonly<T>>,
      ) => void,
      thisArg?: any,
    ): void {
      for (const [key, value] of this) {
        callback.call(thisArg, value, key, this);
      }
    },
    [Symbol.iterator]() {
      return this.entries();
    },
  } as ReadonlyMap<string, Readonly<T>>;
}
