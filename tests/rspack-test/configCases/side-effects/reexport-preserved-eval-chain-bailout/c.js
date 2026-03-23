const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalChainBailout ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalChainBailout = {
		order: []
	});

state.order.push("c");
state.c = "c";

export { value } from "./d";
