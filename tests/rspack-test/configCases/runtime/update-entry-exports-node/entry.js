export const generation = (globalThis.__entryGeneration = (globalThis.__entryGeneration || 0) + 1);
export const state = { generation };
export function render() { return `generation:${generation}`; }
export function reload() {
  const runtime = __TEST_RUNTIME__;
  delete runtime.c[module.id];
  const next = __webpack_require__(module.id);
  if (!runtime.updateEntryExports(module.id, next)) throw Error('Cannot synchronize entry');
}

export function supportsReload() { return typeof __TEST_RUNTIME__.updateEntryExports === "function"; }
