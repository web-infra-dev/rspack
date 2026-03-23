const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalChainRetarget ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalChainRetarget = {
		order: []
	});

state.order.push("a");
state.a = "a";

export { value } from "./b";
