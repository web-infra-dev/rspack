import.meta.webpackHot.accept(["./async-module", "./lazy-module"]);

it("should handle HMR with async chunks in ESM format", async () => {
	// Initial load of async chunks
	const [asyncModule, lazyModule] = await Promise.all([
		import("./async-module"),
		import("./lazy-module")
	]);
	expect(asyncModule.message).toBe("Hello from async module!");
	expect(lazyModule.data.value).toBe(42);

	await NEXT_HMR();
	const [updatedAsyncModule, updatedLazyModule] = await Promise.all([
		import("./async-module"),
		import("./lazy-module")
	]);
	expect(updatedAsyncModule.message).toBe("Updated async module!");
	expect(updatedLazyModule.data.value).toBe(100);
});

it("should support dynamic imports with proper ESM chunk loading", async () => {
	// Test that dynamic imports work correctly with ESM chunk format
	const module = await import("./async-module");
	expect(module.message).toBeDefined();
	expect(typeof module.message).toBe("string");
});
