export function activate(options) {
  const state = globalThis.lazyHmrTerminalStatus;
  state.activated.push(options);
  state.onActivate?.();
  return () => state.disposed++;
}
