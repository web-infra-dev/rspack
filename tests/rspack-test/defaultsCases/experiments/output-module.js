/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "output module",
	options: () => ({ experiments: { outputModule: true } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			+     "outputModule": true,
		`)
};
