import * as values from "./barrel";
export * from "./barrel";

it("should concatenate an export-star chain with a known-empty CommonJS target", () => {
	expect(values.getValue()).toBe(42);
	expect(Object.keys(values)).toEqual(["getValue"]);

	const chunk = __STATS__.chunks.find(chunk => chunk.names.includes("main"));
	expect(chunk.modules.map(module => module.name).sort()).toEqual([
		"./empty.js",
		"./index.js + 2 modules",
		"./sloppy-empty.js"
	]);
});
