const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalChainPartialRetarget ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalChainPartialRetarget = {
		order: []
	});

state.order.push("e");
state.e = "e";

export function effectValue() {
	return "e";
}
