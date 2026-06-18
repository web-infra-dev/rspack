// Impure: an unused deferred pure call to this must be KEPT and run.
export function impureHelper(x) {
	globalThis.__keepAndDropImpureCalls =
		(globalThis.__keepAndDropImpureCalls || 0) + 1;
	return x;
}
