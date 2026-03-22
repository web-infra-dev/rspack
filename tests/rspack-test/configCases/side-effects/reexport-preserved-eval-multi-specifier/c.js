const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalMultiSpecifier ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalMultiSpecifier = {
		order: []
	});

state.c = "c";
state.order.push("c");

export const c = "c";
