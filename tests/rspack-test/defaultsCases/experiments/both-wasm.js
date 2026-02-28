/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "both wasm",
	options: () => ({
		experiments: { syncWebAssembly: true, asyncWebAssembly: true }
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			+     "syncWebAssembly": true,
		`)
};
