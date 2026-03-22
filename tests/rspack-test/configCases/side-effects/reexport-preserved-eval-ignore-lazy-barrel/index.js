import { value } from "./lib";

const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreLazyBarrel ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreLazyBarrel = {
		order: []
	});

state.order.push("index");

it("should not use a lazy barrel sibling to preserve eager evaluation", () => {
	expect(value).toBe("value");
	expect(state.order).toEqual(["mid", "index"]);
});

afterAll(() => {
	delete globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreLazyBarrel;
});
