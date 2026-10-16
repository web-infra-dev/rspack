if (KEEP_BASE) {
	globalThis.__concatenatedPureBaseEffects =
		(globalThis.__concatenatedPureBaseEffects || 0) + 1;
}

export class Base {
	marker() {
		return "UNUSED_CONCATENATED_PURE_ARGUMENT";
	}
}
