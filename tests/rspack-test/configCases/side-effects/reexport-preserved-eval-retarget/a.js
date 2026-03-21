const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalRetarget ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalRetarget = {
		order: []
	});

state.a = "a";
state.order.push("a");

export const a = "a";
