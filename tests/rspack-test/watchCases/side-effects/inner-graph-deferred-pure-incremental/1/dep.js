export function pureFn() {
	globalThis.__innerGraphTracker ??= [];
	globalThis.__innerGraphTracker.push("impure");
	return "impure";
}
