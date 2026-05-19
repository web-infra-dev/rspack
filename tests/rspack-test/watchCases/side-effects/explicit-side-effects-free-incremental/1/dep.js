globalThis.__explicitSideEffectsFreeTracker ??= [];

export function marked() {
	globalThis.__explicitSideEffectsFreeTracker.push("impure");
	return "impure";
}

marked();

export const used = "ok";
