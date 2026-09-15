// Side-effects-free: a deferred pure call to this is droppable on its own.
export function makeConfig() {
	return { id: "CFG" };
}

// NOT side-effects-free: a deferred pure check that resolves to this is impure,
// so an expression calling it is retained for its side effect.
export function register(cfg) {
	globalThis.__deferredPureNestedBindingRegistered = cfg;
	return cfg;
}
