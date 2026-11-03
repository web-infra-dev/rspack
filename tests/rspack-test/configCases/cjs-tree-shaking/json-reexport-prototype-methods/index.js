import imported from "./imported";

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

it("should only retain the receivers' subtrees", () => {
	const source = require("fs").readFileSync(__filename, "utf-8");
	expect(source).not.toContain(["UNUSED", "JSON", "REEXPORT"].join("_"));
});
