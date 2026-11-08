it("should work with dynamic import tree shaking and side effects free", async () => {
	const sideEffects =
		(globalThis.__rspack_dynamic_import_tree_shaking_side_effects_free_executions__ =
			[]);

	const lib = await import("lib");
	const {} = await import("lib");
	let thenCalled = false;
	const libThen = await import("lib").then((m) => {
		thenCalled = true;
	});
	expect(thenCalled).toBe(true);
	let b;
	if (FALSY) {
		b = lib.b;
		b = libThen;
	}
	expect(b).toBeUndefined();
	expect(sideEffects).toEqual(["node_modules/lib/index.js", "node_modules/lib/m.js"]);

	delete globalThis.__rspack_dynamic_import_tree_shaking_side_effects_free_executions__;
});
