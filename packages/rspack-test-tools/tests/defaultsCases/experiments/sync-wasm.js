/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
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
