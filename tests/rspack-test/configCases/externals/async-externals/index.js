const fs = require("fs");
const generated = /** @type {string} */ (fs.readFileSync(__filename, "utf-8"));

import value from "promise-external";
import value2 from "module-promise-external";
import value3 from "object-promise-external";
import request from "import-external";
import request2 from "module-import-external";
import "./module.mjs";

it("should allow async externals", () => {
	expect(value).toBe(42);
	expect(value2).toBe(42);
	expect(value3).toEqual({ default: 42, named: true });
	expect(request).toBe("/hello/world.js");
	expect(request2).toBe("/hello/world.js");
});

it("should allow ProvidePlugin to await async externals", () => {
	// START:ASYNC_PROVIDE
	expect(providedAsyncModule).toMatchObject({
		__esModule: true,
		default: 42,
		named: true
	});
	expect(providedAsyncModuleNamed).toBe(true);
	expect(providedAsyncInlined).toBe(42);
	expect(globalThis.__rspackProvidedAsyncSideEffect).toBe(true);
	// END:ASYNC_PROVIDE
	const generatedPrefix = generated.match(
		/([\s\S]*?)\/\/ START:ASYNC_PROVIDE[\s\S]*?\/\/ END:ASYNC_PROVIDE/
	)[1];
	const declaration = generatedPrefix.indexOf("var providedAsyncInlined =");
	const awaitDependencies = generatedPrefix.indexOf(
		"var __rspack_async_deps",
		declaration
	);
	const inlinedAssignment = generatedPrefix.indexOf(
		"providedAsyncInlined = (/* inlined export .inlined */42)",
		awaitDependencies
	);
	expect(declaration).toBeGreaterThanOrEqual(0);
	expect(awaitDependencies).toBeGreaterThan(declaration);
	expect(inlinedAssignment).toBeGreaterThan(awaitDependencies);
	delete globalThis.__rspackProvidedAsyncSideEffect;
});

it("should allow to catch errors of async externals", () => {
	return expect(() => import("failing-promise-external")).rejects.toEqual(
		expect.objectContaining({
			message: "external reject"
		})
	);
});

it("should allow dynamic import promise externals", () => {
	return import("promise-external").then(module => {
		expect(module).toMatchObject({ default: 42 });
	});
});

it("should allow dynamic import promise externals that are modules", () => {
	return import("module-promise-external").then(module => {
		expect(module).toMatchObject({ default: 42, named: true });
	});
});

it("should allow dynamic import promise externals that are objects", () => {
	return import("object-promise-external").then(module => {
		expect(module).toMatchObject({
			default: { default: 42, named: true },
			named: true
		});
	});
});
