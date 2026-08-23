import * as values from "./barrel";
import "./sloppy-empty";
import "./shadowed-require";
import "./access-exports";
import "./require-access";
import "./dynamic";
import "./top-level-return";
import * as unknown from "./unknown-barrel";
import * as namedValues from "./named-barrel";
import { missing as directMissing } from "./direct-empty";

it("should only concatenate unknown non-ESM modules without CommonJS export access", () => {
	expect(values.getValue()).toBe(42);
	expect(Object.keys(values)).toEqual(["getValue"]);
	expect(Object.keys(unknown)).toEqual([]);
	expect(namedValues.missing).toBeUndefined();
	expect(directMissing).toBeUndefined();
	expect(globalThis.emptyAutoReexportStrictExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportSloppyExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportSloppyThisPreserved).toBe(true);
	expect(globalThis.emptyAutoReexportShadowedRequireExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportAccessExportsExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportRequireAccessExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportDynamicExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportReturnBefore).toBe(true);
	expect(globalThis.emptyAutoReexportReturnAfter).toBeUndefined();
	delete globalThis.emptyAutoReexportStrictExecuted;
	delete globalThis.emptyAutoReexportSloppyExecuted;
	delete globalThis.emptyAutoReexportSloppyThisPreserved;
	delete globalThis.emptyAutoReexportShadowedRequireExecuted;
	delete globalThis.emptyAutoReexportAccessExportsExecuted;
	delete globalThis.emptyAutoReexportRequireAccessExecuted;
	delete globalThis.emptyAutoReexportDynamicExecuted;
	delete globalThis.emptyAutoReexportReturnBefore;

	const empty = __STATS__.modules.find(module => module.name === "./empty.js");
	expect(empty.providedExports).toBe(null);

	const concatenated = __STATS__.modules.find(module =>
		module.modules?.some(nested => nested.name === "./empty.js")
	);
	expect(concatenated.modules.map(module => module.name)).toEqual(
		expect.arrayContaining([
			"./empty.js",
			"./shadowed-require.js"
		])
	);

	for (const name of [
		"./sloppy-empty.js",
		"./access-exports.js",
		"./require-access.js",
		"./required-dep.js",
		"./dynamic.js",
		"./top-level-return.js",
		"./unknown-barrel.js",
		"./named-empty.js",
		"./named-barrel.js",
		"./direct-empty.js"
	]) {
		expect(__STATS__.modules.some(module => module.name === name)).toBe(true);
	}

	const sloppyEmpty = __STATS__.modules.find(
		module => module.name === "./sloppy-empty.js"
	);
	expect(sloppyEmpty.optimizationBailout).toEqual(
		expect.arrayContaining([expect.stringContaining("not in strict mode")])
	);

	const topLevelReturn = __STATS__.modules.find(
		module => module.name === "./top-level-return.js"
	);
	expect(topLevelReturn.optimizationBailout).toEqual(
		expect.arrayContaining([expect.stringContaining("top-level return")])
	);

	const unknownBarrel = __STATS__.modules.find(
		module => module.name === "./unknown-barrel.js"
	);
	expect(unknownBarrel.optimizationBailout).toEqual(
		expect.arrayContaining([
			expect.stringContaining("Reexports in this module do not have a static target")
		])
	);

});
