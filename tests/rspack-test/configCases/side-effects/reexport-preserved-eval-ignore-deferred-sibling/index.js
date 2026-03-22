import defer * as deferredLib from "./lib";
import { value } from "./lib";

const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreDeferredSibling ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreDeferredSibling = {
		order: []
	});

const readDeferredValue = () => deferredLib.value;

state.order.push("index");

it("should not use a deferred sibling to preserve eager evaluation", () => {
	expect(value).toBe("value");
	expect(state.order).toEqual(["lib", "index"]);
	expect(readDeferredValue()).toBe("value");
	expect(state.order).toEqual(["lib", "index"]);
});

afterAll(() => {
	delete globalThis.__configCases_sideEffects_reexportPreservedEvalIgnoreDeferredSibling;
});
