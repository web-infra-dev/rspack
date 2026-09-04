import { getValue, missing } from "./barrel";
import "./sloppy-empty";
import "./shadowed-require";
import "./access-exports";
import "./access-webpack-module";
import "./defined-exports";
import "./require-access";
import "./arguments-access";
import "./mutate-empty";
import "./dynamic";
import "./top-level-return";
import * as namespace from "./namespace-barrel";
import { value as mixedValue } from "./mixed-barrel";

it("concatenates only locally empty CommonJS export-star chains", () => {
	expect(getValue()).toBe(42);
	expect(missing).toBeUndefined();
	expect(mixedValue).toBe(1);
	expect(Object.keys(namespace)).toEqual([]);
	expect(globalThis.emptyAutoReexportMutatedValue).toBe(42);
	expect(globalThis.emptyAutoReexportReturnAfter).toBeUndefined();

	const allModules = __STATS__.modules.flatMap(module => [
		module,
		...(module.modules ?? [])
	]);
	const empty = allModules.find(module => module.name === "./empty.js");
	expect(empty.providedExports).toBe(null);

	const nestedModuleNames = new Set(
		__STATS__.modules.flatMap(module =>
			(module.modules ?? []).map(nested => nested.name)
		)
	);
	for (const name of [
		"./barrel.js",
		"./empty-barrel.js",
		"./empty.js",
		"./shadowed-require.js",
		"./namespace-barrel.js",
		"./mixed-barrel.js",
		"./mixed-empty.js"
	]) {
		expect(nestedModuleNames.has(name)).toBe(true);
	}

	for (const name of [
		"./sloppy-empty.js",
		"./access-exports.js",
		"./access-webpack-module.js",
		"./defined-exports.js",
		"./require-access.js",
		"./arguments-access.js",
		"./mutate-empty.js",
		"./mutated-empty.js",
		"./dynamic.js",
		"./top-level-return.js",
		"./namespace-empty.js",
		"./real-cjs.js"
	]) {
		expect(__STATS__.modules.some(module => module.name === name)).toBe(true);
		expect(nestedModuleNames.has(name)).toBe(false);
	}

	const topLevelReturn = __STATS__.modules.find(
		module => module.name === "./top-level-return.js"
	);
	expect(topLevelReturn.optimizationBailout).toEqual(
		expect.arrayContaining([expect.stringContaining("top-level return")])
	);
	delete globalThis.emptyAutoReexportMutatedValue;
	delete globalThis.emptyAutoReexportReturnAfter;
});
