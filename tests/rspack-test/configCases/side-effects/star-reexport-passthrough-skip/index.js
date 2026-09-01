import * as ns from "lib";
import * as defaultNs from "lib-default";
import * as defaultNamespaceNs from "lib-default-namespace";
import { foo, bar } from "./named.js";

const SKIP = "(skipped side-effect-free modules)";

it("should resolve named and namespace imports through a pure star passthrough chain", () => {
	expect(foo()).toBe(1);
	expect(bar()).toBe(2);
	expect(ns.foo()).toBe(1);
	expect(ns.bar()).toBe(2);
	expect(ns.baz()).toBe(3);
	expect(Object.keys(ns).sort()).toEqual(["bar", "baz", "foo"]);
});

it("should not collapse a passthrough target with its own default export", () => {
	expect(defaultNs.value()).toBe("value");
	expect(Object.keys(defaultNs)).toEqual(["value"]);
	expect(defaultNamespaceNs.value()).toBe("value");
	expect(Object.keys(defaultNamespaceNs)).toEqual(["value"]);

	for (const packageName of ["lib-default", "lib-default-namespace"]) {
		const real = __STATS__.modules.find((module) =>
			module.name.endsWith(`${packageName}/real.js`)
		);
		const skippedFrom = new Set(
			real.reasons
				.filter((reason) => reason.explanation === SKIP)
				.map((reason) => reason.moduleName)
		);
		expect(skippedFrom.has(`./node_modules/${packageName}/index.js`)).toBe(false);
	}
});

it("should repoint consumers past the passthrough chain onto the real module", () => {
	const real = __STATS__.modules.find((m) => m.name.endsWith("lib/real.js"));
	const skippedFrom = new Set(
		real.reasons.filter((r) => r.explanation === SKIP).map((r) => r.moduleName)
	);
	expect(skippedFrom.has("./index.js")).toBe(true);
	expect(skippedFrom.has("./named.js")).toBe(true);
	expect(skippedFrom.has("./node_modules/lib/a.js")).toBe(true);
});

it("should leave the intermediate star passthrough modules unused", () => {
	const intermediates = __STATS__.modules.filter(
		(m) => m.name.endsWith("lib/a.js") || m.name.endsWith("lib/b.js")
	);
	expect(intermediates).toHaveLength(2);
	for (const module of intermediates) {
		expect(module.usedExports).toBe(false);
	}
});
