/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "true runtimeChunk",
	options: () => ({ optimization: { runtimeChunk: true } }),
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
};
