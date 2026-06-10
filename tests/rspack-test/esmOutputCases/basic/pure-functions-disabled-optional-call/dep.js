export function sideEffect(label) {
  (globalThis.__PURE_FUNCTION_DISABLED_OPTIONAL_CALLS__ ||= []).push(label);
  return 1;
}
