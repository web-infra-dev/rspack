globalThis.__CJS_NESTED_FREE_READ_COUNT__ = 0;
Object.defineProperty(globalThis, 'missingNested', {
  configurable: true,
  get() {
    globalThis.__CJS_NESTED_FREE_READ_COUNT__ += 1;
    return 9;
  },
});
