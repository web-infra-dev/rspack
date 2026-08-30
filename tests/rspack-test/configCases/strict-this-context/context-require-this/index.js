// Use variables to force context modules
var aName = "a";
var libName = "index";

it("should respect strictThisContextOnImports for member call via require context module", () => {
	let m = require("./dir4/" + aName);
	expect(m.f()).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 1 : undefined);
	expect(m.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	let m2 = require("./dir4/lib/" + libName);
	expect(m2.b.f()).toBe(1);
	expect(m2.b.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["b", "f", "usedExports"]);
	expect(m2.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["b", "f", "usedExports"]);
})

it("should always correctly handle this for exportsType is DefaultWithNamed and DefaultOnly via require context module", () => {
	const cjsName = "cjs";
	const cjs = require("./modules/" + cjsName);
	expect(cjs.that().value).toBe(42);
	const dataName = "data.json";
	const json = require("./modules/" + dataName);
	expect(json.map(d => d * 2)).toEqual([2, 4, 6]);
})
