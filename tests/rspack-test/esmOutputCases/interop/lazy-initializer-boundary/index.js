function load() {
	return require("./wrapped.js");
}

it("keeps scope-hoisted dependencies behind a CommonJS initializer", () => {
	expect(globalThis.__esmLazyEvaluationLog).toBeUndefined();

	expect(load().value).toBe(42);
	expect(globalThis.__esmLazyEvaluationLog).toEqual(["child", "wrapped"]);

	expect(load().value).toBe(42);
	expect(globalThis.__esmLazyEvaluationLog).toEqual(["child", "wrapped"]);
});
