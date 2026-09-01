import * as ns from "lib";
import * as reexportedNs from "./reexport.js";
import { foo, bar } from "lib";
import { foo as reexportedFoo } from "./reexport.js";

const SKIP = "(skipped side-effect-free modules)";

it("should resolve imports through a multi-star barrel", () => {
	expect(foo()).toBe(1);
	expect(bar()).toBe(2);
	expect(reexportedFoo()).toBe(1);
	expect(ns.foo()).toBe(1);
	expect(ns.bar()).toBe(2);
	expect(Object.keys(ns).sort()).toEqual(["bar", "foo"]);
	expect(reexportedNs.foo()).toBe(1);
	expect(reexportedNs.bar()).toBe(2);
	expect(Object.keys(reexportedNs).sort()).toEqual(["bar", "foo"]);
});

it("should skip only the single-star sub-chains", () => {
	const reasons = (suffix) => {
		const module = __STATS__.modules.find((item) => item.name.endsWith(suffix));
		return new Set(
			module.reasons
				.filter((reason) => reason.explanation === SKIP)
				.map((reason) => reason.moduleName)
		);
	};
	expect(reasons("lib/real-a.js").has("./node_modules/lib/index.js")).toBe(true);
	expect(reasons("lib/real-b.js").has("./node_modules/lib/index.js")).toBe(true);
	expect(reasons("lib/real-a.js").has("./reexport.js")).toBe(false);
	expect(reasons("lib/real-b.js").has("./reexport.js")).toBe(false);
	const barrel = __STATS__.modules.find((item) => item.name.endsWith("lib/index.js"));
	expect(barrel.reasons.some((reason) => reason.moduleName === "./reexport.js")).toBe(true);
});

it("should leave the intermediate single-star passthroughs unused", () => {
	for (const suffix of ["lib/a.js", "lib/b.js"]) {
		expect(__STATS__.modules.find((module) => module.name.endsWith(suffix)).usedExports).toBe(false);
	}
});
