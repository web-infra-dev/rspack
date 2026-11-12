/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
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
