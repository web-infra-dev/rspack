const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreLazyBarrel ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreLazyBarrel = {
		order: []
	});

state.order.push("mid");

export { value } from "./value";
