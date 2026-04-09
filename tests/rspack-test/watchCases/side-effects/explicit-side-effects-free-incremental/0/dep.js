globalThis.__explicitSideEffectsFreeTracker ??= [];

/*#__NO_SIDE_EFFECTS__*/
export function marked() {
	return "pure";
}

marked();

export const used = "ok";
