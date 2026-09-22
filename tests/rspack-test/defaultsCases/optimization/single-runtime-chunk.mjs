/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
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
};
