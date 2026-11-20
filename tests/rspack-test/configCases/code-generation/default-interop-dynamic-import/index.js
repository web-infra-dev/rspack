import * as namedOnly from "./named-only";
import * as withDefault from "./with-default";

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
