// The real, side-effecting function at the end of a multi-hop re-export chain.
// It is NOT side-effects-free, so a deferred pure check that resolves to it must
// conclude "impure" and keep the call alive (and not leave an undefined(...)
// partial-sever).
export function impl(value) {
	globalThis.__deferredPureReexportChainCalls =
		(globalThis.__deferredPureReexportChainCalls || 0) + 1;
	return value;
}
