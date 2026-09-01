import * as ns from "lib";
import { foo } from "lib";

const SKIP = "(skipped side-effect-free modules)";

it("should resolve named and namespace imports through a star and named barrel", () => {
	expect(foo()).toBe(1);
	expect(ns.foo()).toBe(1);
	expect(ns.c()).toBe(3);
	expect(Object.keys(ns).sort()).toEqual(["c", "foo"]);
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
	expect(reasons("lib/real-c.js").has("./node_modules/lib/index.js")).toBe(true);
});

it("should leave the intermediate single-star passthroughs unused", () => {
	for (const suffix of ["lib/a.js", "lib/c.js"]) {
		expect(__STATS__.modules.find((module) => module.name.endsWith(suffix)).usedExports).toBe(false);
	}
});
