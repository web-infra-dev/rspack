it("should static analyze require destructuring assignment", () => {
	const { a, usedExports } = require("./module");
	expect(a).toBe("a");
	expect(usedExports).toEqual(["a", "usedExports"]);
	const { b, usedExports: usedExports2 } = module.require("./module?2");
	expect(b).toBe("b");
	expect(usedExports2).toEqual(["b", "usedExports"]);
});

it("should static analyze require destructuring assignment from a variable", () => {
	const m = require("./module?3");
	const { b, usedExports } = m;
	expect(b).toBe("b");
	expect(usedExports).toEqual(["b", "usedExports"]);
});

it("should support require context destructuring assignment", () => {
	const file = "a";
	const { a, usedExports } = require(`./dir/${file}.js`);
	expect(a).toBe("a/a");
	expect(usedExports).toEqual(["a", "usedExports"]);
});

it("should not analyze destructuring member access", () => {
	const m = require("./nested");
	const { a, usedExports } = m.inner;
	expect(a).toBe("a");
	expect(usedExports).toEqual(true);
});
