it("should static analyze require member access", () => {
	const m = require("../require-destructuring/module");
	expect(m.a).toBe("a");
	expect(m.usedExports).toEqual(["a", "usedExports"]);
	const m2 = module.require("../require-destructuring/module?2");
	expect(m2.b).toBe("b");
	expect(m2.usedExports).toEqual(["b", "usedExports"]);
});

it("should support require context member access", () => {
	const file = "a";
	const m = require(`../require-destructuring/dir/${file}.js`);
	expect(m.a).toBe("a/a");
	expect(m.usedExports).toEqual(["a", "usedExports"]);
});

it("should not require user defined require", () => {
	const require = (s) => s;
	expect(require("./module?3")).toBe("./module?3");
	expect(__STATS__.modules.some(m => m.name.endsWith("require-destructuring/module.js?3"))).toBe(false);
	const module = { require: (s) => s };
	expect(module.require("./module?4")).toBe("./module?4");
	expect(__STATS__.modules.some(m => m.name.endsWith("require-destructuring/module.js?4"))).toBe(false);
});
