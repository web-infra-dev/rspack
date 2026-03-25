it('should evaluate `typeof import.meta.resolve` to "function"', () => {
	expect(typeof import.meta.resolve).toBe("function");
});

it("should evaluate `import.meta.resolve` as a truthy expression", () => {
	let id;

	if (import.meta.resolve) {
		id = import.meta.resolve("./a.js");
	}

	expect(id).toBe(require.resolve("./a.js"));

	const ternaryId = import.meta.resolve
		? import.meta.resolve("./b.js")
		: null;

	expect(ternaryId).toBe(require.resolve("./b.js"));
});

it("should resolve statically analyzable string requests", () => {
	expect(import.meta.resolve("./a.js")).toBe(require.resolve("./a.js"));
	expect(import.meta.resolve("./" + "b.js")).toBe(require.resolve("./b.js"));
	expect(import.meta.resolve(`./${"a"}.js`)).toBe(require.resolve("./a.js"));
});

it("should rewrite string branches inside conditional expressions", () => {
	const flag = Math.random() > 0.5;
	const aId = require.resolve("./a.js");
	const bId = require.resolve("./b.js");

	expect(import.meta.resolve(flag ? "./a.js" : "./b.js")).toBe(flag ? aId : bId);
	expect(import.meta.resolve(flag ? "./a.js" : 42)).toBe(flag ? aId : 42);
});
