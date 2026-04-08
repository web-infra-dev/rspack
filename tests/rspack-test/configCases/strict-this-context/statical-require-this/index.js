it("should respect strictThisContextOnImports for member call", () => {
	let m = require("./dir4/a");
	expect(m.f()).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 1 : undefined);
	expect(m.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	let m2 = require("./dir4/lib");
	expect(m2.b.f()).toBe(1);
	expect(m2.b.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	expect(m2.usedExports).toEqual(["b", "usedExports"]);
})

it("should always correctly handle this for exportsType is DefaultWithNamed and DefaultOnly", () => {
	const cjs = require("./cjs");
	expect(cjs.that().value).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 42 : undefined);
	const json = require("./data.json");
	expect(json.map(d => d * 2)).toEqual([2, 4, 6]);
})
