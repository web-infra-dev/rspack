it("should work with cjs tree shaking and side effects free", () => {
	const sideEffects =
		(globalThis.__rspack_cjs_tree_shaking_side_effects_free_executions__ =
			[]);

	const lib = require("lib");
	const {} = require("lib");
	const unusedNewRequire = new require("lib");
	const {} = new require("lib");
	let b;
	if (FALSY) {
		b = lib.b;
		b = unusedNewRequire.b;
	}
	expect(b).toBeUndefined();
	expect(sideEffects).toEqual(["node_modules/lib/index.js", "node_modules/lib/m.js"]);

	delete globalThis.__rspack_cjs_tree_shaking_side_effects_free_executions__;
});
