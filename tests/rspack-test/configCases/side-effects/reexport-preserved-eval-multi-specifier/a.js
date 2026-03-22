const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalMultiSpecifier ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalMultiSpecifier = {
		order: []
	});

state.a = "a";
state.order.push("a");

export const a = "a";
