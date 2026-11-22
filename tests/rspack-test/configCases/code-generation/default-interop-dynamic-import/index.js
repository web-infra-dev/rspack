import * as namedOnly from "./named-only";
import * as withDefault from "./with-default";
import reexportedDefault from "./reexport-default";
import * as withRequire from "./with-require";
import { answer as reexportedAnswer } from "./reexport-star";

const fs = require("fs");
const path = require("path");

const importVar = /__rspack_import_\d+ = /;
const defaultInteropVar = /__rspack_import_\d+_default\b/;

it("should skip the default interop getter when only import() reads the default export", async () => {
	expect(namedOnly.answer).toBe(42);
	expect((await namedOnly.lazy()).answer).toBe(42);
	expect((await namedOnly.eager()).answer).toBe(42);
	expect((await namedOnly.weak()).answer).toBe(42);

	const content = fs.readFileSync(
		path.join(__dirname, "named-only.js"),
		"utf-8"
	);
	expect(content).toMatch(importVar);
	expect(content).not.toMatch(defaultInteropVar);
});

it("should keep the default interop getter for a default import next to import()", async () => {
	expect(withDefault.answer).toBe(42);
	expect((await withDefault.lazy()).answer).toBe(42);

	const content = fs.readFileSync(
		path.join(__dirname, "with-default.js"),
		"utf-8"
	);
	expect(content).toMatch(importVar);
	expect(content).toMatch(defaultInteropVar);
});

it("should keep the default interop getter for a default reexport", () => {
	expect(reexportedDefault.answer).toBe(42);

	const content = fs.readFileSync(
		path.join(__dirname, "reexport-default.js"),
		"utf-8"
	);
	expect(content).toMatch(defaultInteropVar);
});

it("should skip the default interop getter when only require() reads the module", () => {
	expect(withRequire.answer).toBe(42);
	expect(withRequire.required().answer).toBe(42);
	expect(withRequire.requiredAnswer()).toBe(42);

	const content = fs.readFileSync(
		path.join(__dirname, "with-require.js"),
		"utf-8"
	);
	expect(content).toMatch(importVar);
	expect(content).not.toMatch(defaultInteropVar);
});

it("should skip the default interop getter for a star reexport", () => {
	expect(reexportedAnswer).toBe(42);

	const content = fs.readFileSync(
		path.join(__dirname, "reexport-star.js"),
		"utf-8"
	);
	expect(content).toMatch(importVar);
	expect(content).not.toMatch(defaultInteropVar);
});
