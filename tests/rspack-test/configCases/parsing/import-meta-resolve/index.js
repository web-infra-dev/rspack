it('should evaluate `typeof import.meta.resolve` to "function"', () => {
	expect(typeof import.meta.resolve).toBe("function");
});

it("should evaluate `import.meta.resolve` as a truthy expression", () => {
	let id;

	if (import.meta.resolve) {
		id = import.meta.resolve("./a.js");
	}

	expect(id).toBe(new URL("./a.js", import.meta.url).href);

	const ternaryId = import.meta.resolve
		? import.meta.resolve("./b.js")
		: null;

	expect(ternaryId).toBe(new URL("./b.js", import.meta.url).href);
});

it("should resolve statically analyzable string requests", () => {
	expect(import.meta.resolve("./a.js")).toBe(new URL("./a.js", import.meta.url).href);
	expect(import.meta.resolve("./" + "b.js")).toBe(new URL("./b.js", import.meta.url).href);
	expect(import.meta.resolve(`./${"a"}.js`)).toBe(new URL("./a.js", import.meta.url).href);
});
