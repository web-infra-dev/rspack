import * as values from "./barrel";
import "./sloppy-empty";
import "./shadowed-require";
import "./access-exports";
import "./require-access";
import "./mutate-empty";
import "./dynamic";
import "./top-level-return";
import * as unknown from "./unknown-barrel";
import * as namedValues from "./named-barrel";
import { missing as directMissing } from "./direct-empty";
import { value as mutatedValue } from "./mutated-empty";
import { value as amdRequireValue } from "./amd-require-empty";
import { __esModule as directEsModule } from "./es-module-empty";
import * as directNamespace from "./namespace-empty";
import directDefault from "./default-empty";
import * as namespaceMember from "./namespace-member-empty";
import * as _unusedNamespace from "./unused-namespace-empty";
import _unusedDefault from "./unused-default-empty";
import { default as _unusedNamedDefault } from "./unused-named-default-empty";
import {
	emptyNamespace,
	emptyDefault,
	reexportedMissing,
	reexportedEsModule
} from "./reexports";
const dynamicEmpty = import(
	/* webpackMode: "eager" */ "./dynamic-import-empty"
);

it("should only concatenate unknown non-ESM modules without CommonJS export access", async () => {
	const dynamicNamespace = await dynamicEmpty;
	expect(values.getValue()).toBe(42);
	expect(Object.keys(values)).toEqual(["getValue"]);
	expect(Object.keys(unknown)).toEqual([]);
	expect(namedValues.missing).toBeUndefined();
	expect(directMissing).toBeUndefined();
	expect(mutatedValue).toBe(42);
	expect(amdRequireValue).toBe(42);
	expect(directEsModule).toBe(true);
	expect(directNamespace.default).toEqual({});
	expect(directDefault).toEqual({});
	expect(namespaceMember.missing).toBeUndefined();
	expect(emptyNamespace.default).toEqual({});
	expect(emptyDefault).toEqual({});
	expect(reexportedMissing).toBeUndefined();
	expect(reexportedEsModule).toBe(true);
	expect(dynamicNamespace.default).toEqual({});
	expect(globalThis.emptyAutoReexportStrictExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportSloppyExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportSloppyThisPreserved).toBe(true);
	expect(globalThis.emptyAutoReexportShadowedRequireExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportAccessExportsExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportRequireAccessExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportMutatedExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportMutatorExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportAmdRequireExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportDynamicExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportReturnBefore).toBe(true);
	expect(globalThis.emptyAutoReexportReturnAfter).toBeUndefined();
	expect(globalThis.emptyAutoReexportNamespaceExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportDefaultExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportNamespaceMemberExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportUnusedNamespaceExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportUnusedDefaultExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportUnusedNamedDefaultExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportNamespaceReexportExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportDefaultReexportExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportNamedReexportExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportEsModuleExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportEsModuleReexportExecuted).toBe(true);
	expect(globalThis.emptyAutoReexportDynamicImportExecuted).toBe(true);
	delete globalThis.emptyAutoReexportStrictExecuted;
	delete globalThis.emptyAutoReexportSloppyExecuted;
	delete globalThis.emptyAutoReexportSloppyThisPreserved;
	delete globalThis.emptyAutoReexportShadowedRequireExecuted;
	delete globalThis.emptyAutoReexportAccessExportsExecuted;
	delete globalThis.emptyAutoReexportRequireAccessExecuted;
	delete globalThis.emptyAutoReexportMutatedExecuted;
	delete globalThis.emptyAutoReexportMutatorExecuted;
	delete globalThis.emptyAutoReexportAmdRequireExecuted;
	delete globalThis.emptyAutoReexportDynamicExecuted;
	delete globalThis.emptyAutoReexportReturnBefore;
	delete globalThis.emptyAutoReexportNamespaceExecuted;
	delete globalThis.emptyAutoReexportDefaultExecuted;
	delete globalThis.emptyAutoReexportNamespaceMemberExecuted;
	delete globalThis.emptyAutoReexportUnusedNamespaceExecuted;
	delete globalThis.emptyAutoReexportUnusedDefaultExecuted;
	delete globalThis.emptyAutoReexportUnusedNamedDefaultExecuted;
	delete globalThis.emptyAutoReexportNamespaceReexportExecuted;
	delete globalThis.emptyAutoReexportDefaultReexportExecuted;
	delete globalThis.emptyAutoReexportNamedReexportExecuted;
	delete globalThis.emptyAutoReexportEsModuleExecuted;
	delete globalThis.emptyAutoReexportEsModuleReexportExecuted;
	delete globalThis.emptyAutoReexportDynamicImportExecuted;

	const empty = __STATS__.modules.find(module => module.name === "./empty.js");
	expect(empty.providedExports).toBe(null);

	const concatenated = __STATS__.modules.find(module =>
		module.modules?.some(nested => nested.name === "./empty.js")
	);
	expect(concatenated.modules.map(module => module.name)).toEqual(
		expect.arrayContaining([
			"./empty.js",
			"./shadowed-require.js",
			"./require-access.js",
			"./mutate-empty.js",
			"./direct-empty.js",
			"./named-empty.js",
			"./named-barrel.js",
			"./named-reexport-empty.js"
		])
	);
	const nestedModuleNames = new Set(
		__STATS__.modules.flatMap(module =>
			(module.modules ?? []).map(nested => nested.name)
		)
	);

	for (const name of [
		"./sloppy-empty.js",
		"./access-exports.js",
		"./required-dep.js",
		"./mutated-empty.js",
		"./amd-require-empty.js",
		"./dynamic.js",
		"./top-level-return.js",
		"./unknown-barrel.js",
		"./namespace-empty.js",
		"./default-empty.js",
		"./namespace-member-empty.js",
		"./unused-namespace-empty.js",
		"./unused-default-empty.js",
		"./unused-named-default-empty.js",
		"./namespace-reexport-empty.js",
		"./default-reexport-empty.js",
		"./es-module-empty.js",
		"./es-module-reexport-empty.js",
		"./dynamic-import-empty.js"
	]) {
		expect(__STATS__.modules.some(module => module.name === name)).toBe(true);
		expect(nestedModuleNames.has(name)).toBe(false);
	}

	const sloppyEmpty = __STATS__.modules.find(
		module => module.name === "./sloppy-empty.js"
	);
	expect(sloppyEmpty.optimizationBailout).toEqual(
		expect.arrayContaining([
			expect.stringContaining("not an ECMAScript module")
		])
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
