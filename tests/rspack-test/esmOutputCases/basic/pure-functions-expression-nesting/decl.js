export function a(value, label) {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push(label);
  return value;
}

export function b(label, value) {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push(label);
  return value;
}
