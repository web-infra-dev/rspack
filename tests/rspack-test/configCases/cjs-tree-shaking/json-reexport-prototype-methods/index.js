import imported from "./imported";
import optionalImported from "./optional-imported";
import { list as esmNamed } from "./esm-named";
import * as esmNamespace from "./esm-namespace";
import { upperCase } from "./strict.mjs";
import "./weak";

const contextName = ["array"][0];

// Keep each JSON module separate: observing a whole value in another test would
// preserve its data and hide incorrect reference tracking.
it("should preserve an imported CommonJS reexport's array receiver", () => {
	expect(imported.map(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
});

it("should preserve a directly required CommonJS reexport's array receiver", () => {
	expect(require("./required").join(",")).toBe("md,markdown");
});

it("should preserve an assigned CommonJS reexport's array receiver", () => {
	const assigned = require("./assigned");
	expect(assigned.includes("markdown")).toBe(true);
});

it("should preserve a dynamically imported CommonJS reexport's array receiver", async () => {
	await import("./dynamic").then(module => {
		expect(module.default.map(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
	});
});

it("should preserve an eagerly imported CommonJS reexport's array receiver", async () => {
	await import(/* webpackMode: "eager" */ "./eager").then(module => {
		expect(module.default.map(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
	});
});

it("should preserve only the nested receivers of prototype methods", () => {
	expect(require("./nested").list.includes("markdown")).toBe(true);
	expect(require("./nested").object.hasOwnProperty("kept")).toBe(true);
});

it("should preserve a receiver reexported from a nested property", () => {
	expect(require("./prefixed").join(",")).toBe("md,markdown");
});

it("should distinguish locally added exports from runtime properties", () => {
	const augmented = require("./augmented");
	expect(augmented.local).toBe(7);
	expect(augmented.localFunction()).toBe(42);
	expect(augmented.list.includes("markdown")).toBe(true);
});

it("should preserve receivers of optional calls", () => {
	expect(require("./optional-required").map?.(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
	expect(optionalImported.map?.(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
	expect(require("./optional-nested").list.includes?.("markdown")).toBe(true);
});

it("should preserve a JSON receiver reexported through ESM", () => {
	expect(esmNamed.map(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
	expect(esmNamespace.list.join(",")).toBe("md,markdown");
});

it("should preserve a CommonJS reexport's array receiver imported by strict ESM", () => {
	expect(upperCase()).toEqual(["MD", "MARKDOWN"]);
});

it("should preserve a weakly imported CommonJS reexport's array receiver", async () => {
	await import(/* webpackMode: "weak" */ "./weak").then(module => {
		expect(module.default.map(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
	});
});

it("should preserve a context imported CommonJS reexport's array receiver", async () => {
	const module = await import(`./context-import/${contextName}`);
	expect(module.default.map(value => value.toUpperCase())).toEqual(["MD", "MARKDOWN"]);
});

it("should preserve a context required CommonJS reexport's array receiver", () => {
	expect(require(`./context-require/${contextName}`).join(",")).toBe("md,markdown");
});

it("should only retain the receivers' subtrees", () => {
	expect(globalThis.__prototypeMethodsUsedExports.nested).toEqual(["list", "object"]);
	expect(globalThis.__prototypeMethodsUsedExports.augmented).toEqual(["list", "local", "localFunction"]);
	expect(globalThis.__prototypeMethodsUsedExports.optionalNested).toEqual(["list"]);
});
