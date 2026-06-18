// Side-effects-free: an unused deferred pure call to this must be fully dropped,
// and (being its only consumer) this whole module must be tree-shaken away.
export function pureHelper(x) {
	return x + 1;
}
