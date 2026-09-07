import "./setup.js";
import defer * as direct from "./dep.js";
import { reexported } from "./barrel.js";

it("should preserve the identity of a deferred namespace re-exported through a side-effect-free barrel", () => {
	expect(globalThis.deferReexportEvaluationCount).toBe(0);
	expect(direct).toBe(reexported);
	expect(globalThis.deferReexportEvaluationCount).toBe(0);
	expect(direct.value).toBe(42);
	expect(globalThis.deferReexportEvaluationCount).toBe(1);
});
