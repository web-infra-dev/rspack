export function context() {}

it("should preserve an exported context binding", () => {
	expect(context.name).toBe(
		globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK ? "index_context" : "context",
	);
});
