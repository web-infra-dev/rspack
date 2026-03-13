it("should static analyze require member access", () => {
	const m = require("../require-destructuring/module");
	expect(m.a).toBe("a");
	expect(m.usedExports).toEqual(["a", "usedExports"]);
});

it("should support require context member access", () => {
	const file = "a";
	const m = require(`../require-destructuring/dir/${file}.js`);
	expect(m.a).toBe("a/a");
	expect(m.usedExports).toEqual(["a", "usedExports"]);
});
