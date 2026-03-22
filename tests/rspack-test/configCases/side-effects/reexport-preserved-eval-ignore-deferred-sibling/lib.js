const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreDeferredSibling ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreDeferredSibling = {
		order: []
	});

state.order.push("lib");

export { value } from "./value";
