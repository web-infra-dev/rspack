/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "new cache",
	options: () => ({ experiments: { newCache: true } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "newCache": false,
			+     "newCache": Object {
			+       "codeGeneration": true,
			+       "minimize": true,
			+       "module": true,
			+     },
		`)
};
