/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "array defaults",
	options: () => ({
		output: {
			enabledChunkLoadingTypes: ["require", "..."],
			enabledWasmLoadingTypes: ["...", "async-node"]
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+       "require",
		@@ ... @@
		+       "async-node",
	`)
};
