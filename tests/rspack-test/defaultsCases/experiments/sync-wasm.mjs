/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "sync wasm",
	options: () => ({ experiments: { syncWebAssembly: true } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+     "syncWebAssembly": true,
	`)
};
