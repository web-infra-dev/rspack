const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalRetarget ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalRetarget = {
		order: []
	});

state.c = "c";
state.order.push("c");

export const c = "c";
