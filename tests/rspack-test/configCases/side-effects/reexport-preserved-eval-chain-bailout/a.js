const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalChainBailout ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalChainBailout = {
		order: []
	});

state.order.push("a");
state.a = "a";

export { value } from "./b";
