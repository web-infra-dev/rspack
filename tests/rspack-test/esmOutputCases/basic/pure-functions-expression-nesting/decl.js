export function pureFn(label) {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push(label);
  return 1;
}

export function a(value) {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("PURE_NESTING_FN_A_MARKER");
  return value;
}

export function b() {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("PURE_NESTING_FN_B_MARKER");
  return 1;
}
