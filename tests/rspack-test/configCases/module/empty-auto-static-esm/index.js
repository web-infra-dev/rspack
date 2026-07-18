import * as inferredNamespace from "./static-empty";
import importedCommonJs from "./non-empty-cjs";

it("shares one empty ESM namespace between import and require", () => {
	const inferredRequired = require("./static-empty");
	const explicitRequired = require("./explicit-empty-esm");

	expect(Object.keys(inferredNamespace)).toEqual([]);
	expect(Object.keys(inferredRequired)).toEqual([]);
	expect(inferredRequired.__esModule).toBe(explicitRequired.__esModule);
});

it("does not promote a CommonJS-only empty module", () => {
	expect(require("./cjs-only-empty").__esModule).toBeUndefined();
});

it("does not promote a dynamic-import-only empty module", async () => {
	const namespace = await import("./dynamic-only-empty");

	expect(Object.keys(namespace)).toEqual(["default"]);
	expect(namespace.default).toEqual({});
});

it("does not change non-empty CommonJS modules", () => {
	expect(importedCommonJs).toBe("commonjs");
	expect(require("./non-empty-cjs")).toBe("commonjs");
});
