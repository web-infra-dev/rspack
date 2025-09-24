defineDefaultsCase(Utils.casename(__filename), {
	description: "single runtimeChunk",
	options: () => ({ optimization: { runtimeChunk: "single" } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "runtimeChunk": false,
		+     "runtimeChunk": Object {
		+       "name": "single",
		+     },
	`)
});
