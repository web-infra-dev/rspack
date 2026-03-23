const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalChainPartialRetarget ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalChainPartialRetarget = {
		order: []
	});

state.order.push("a");
state.a = "a";

export { pureValue, effectValue } from "./b";
