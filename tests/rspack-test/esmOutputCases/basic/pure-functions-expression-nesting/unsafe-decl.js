export function unsafePureFn(label) {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push(label);
  return 1;
}
