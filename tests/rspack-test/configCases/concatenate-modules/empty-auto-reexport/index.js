import { value } from "./barrel";
import defaultValue from "./empty.js?default";
import * as directNamespace from "./empty.js?namespace";
import "./amd-exports";
import "./amd-require";
import "./sloppy-empty";
import "./shadowed-require";
import "./access-exports";
import "./access-webpack-module";
import "./defined-exports";
import "./require-access";
import "./arguments-access";
import "./empty.js?dynamic";
import "./top-level-return";
import * as chainedNamespace from "./namespace-chain-barrel";
import { value as mixedValue } from "./mixed-barrel";

const stats = __STATS__;
const effects = [
	globalThis.emptyAutoAmdDefine,
	globalThis.emptyAutoReexportReturnAfter
];
const amdRequire = globalThis.emptyAutoAmdRequire;
const resolved = globalThis.emptyAutoResolve;
delete globalThis.emptyAutoAmdDefine;
delete globalThis.emptyAutoAmdRequire;
delete globalThis.emptyAutoResolve;
delete globalThis.emptyAutoReexportReturnAfter;

const nestedModules = stats.modules.flatMap(module => module.modules ?? []);
const nestedModuleNames = new Set(nestedModules.map(module => module.name));

it("concatenates only locally empty CommonJS export-star chains", () => {
	expect(value).toBe(42);
	expect(defaultValue).toEqual({});
	expect(defaultValue.__esModule).toBeUndefined();
	expect(directNamespace.default).toEqual({});
	expect(directNamespace.default.__esModule).toBeUndefined();
	expect(effects).toEqual([42, undefined]);
	expect(mixedValue).toBe(1);
	expect(resolved).toEqual([
		"./empty.js?require-target",
		"./empty.js?require-target"
	]);
	const empty = nestedModules.find(module => module.name === "./empty.js");
	expect(empty.providedExports).toBe(null);

	for (const name of [
		"./barrel.js",
		"./empty-barrel.js",
		"./empty.js",
		"./shadowed-require.js",
		"./empty.js?mixed",
		"./mixed-barrel.js"
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
		"./empty.js?require-target",
		"./empty.js?dynamic",
		"./top-level-return.js",
		"./real-cjs.js",
		"./empty.js?default",
		"./empty.js?namespace",
		"./empty.js?dynamic-import",
		"./amd-exports.js",
		"./amd-require.js"
	]) {
		expect(stats.modules.some(module => module.name === name)).toBe(true);
		expect(nestedModuleNames.has(name)).toBe(false);
	}

	const topLevelReturn = stats.modules.find(
		module => module.name === "./top-level-return.js"
	);
	expect(topLevelReturn.optimizationBailout).toEqual(
		expect.arrayContaining([expect.stringContaining("top-level return")])
	);
});

it("omits missing exports from a namespace observed through multiple star reexports", () => {
	// A missing binding reads as undefined without becoming an own namespace property.
	expect(Object.keys(chainedNamespace)).toEqual(["explicitMissing"]);
	expect(Object.hasOwn(chainedNamespace, "explicitMissing")).toBe(true);
	expect(chainedNamespace.explicitMissing).toBeUndefined();
	expect(Object.hasOwn(chainedNamespace, "value")).toBe(false);
	expect(chainedNamespace.value).toBeUndefined();
	const empty = nestedModules.find(
		module => module.name === "./empty.js?namespace-chain"
	);
	expect(empty).toBeDefined();
	expect(empty.providedExports).toBe(null);
	expect(nestedModuleNames.has(empty.name)).toBe(true);
});

it("keeps the CommonJS wrapper for dynamic import", async () => {
	const namespace = await import("./empty.js?dynamic-import");
	expect(namespace.default).toEqual({});
	expect(namespace.default.__esModule).toBeUndefined();
	expect(await amdRequire).toBe(true);
});
