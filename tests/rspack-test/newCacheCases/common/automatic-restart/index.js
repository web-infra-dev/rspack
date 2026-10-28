import value from "./value";

let hooks = 0;
beforeEach(() => {
	hooks++;
});
afterEach(() => {
	expect(hooks).toBe(1);
	hooks--;
});

it("should execute both cold and warm bundles", async () => {
	expect(hooks).toBe(1);
	recordCompiler(COMPILER_INDEX);
	expect(value).toBe(42);
	expect((await import("./async")).default).toBe("async");
	expect(COMPILER_INDEX).toBeLessThanOrEqual(1);
});
