defineDefaultsCase(Utils.casename(__filename), {
	description: "multiple runtimeChunk",
	options: () => ({ optimization: { runtimeChunk: "multiple" } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "runtimeChunk": false,
		+     "runtimeChunk": Object {
		+       "name": "multiple",
		+     },
	`)
});
