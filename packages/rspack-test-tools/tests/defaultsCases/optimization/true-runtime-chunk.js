module.exports = {
	description: "true runtimeChunk",
	options: () => ({ optimization: { runtimeChunk: true } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "runtimeChunk": false,
		+     "runtimeChunk": Object {
		+       "name": [Function name],
		+     },
	`)
};
