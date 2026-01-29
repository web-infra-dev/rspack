/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "experiments.futureDefaults",
	options: () => ({
		experiments: {
			futureDefaults: true
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "futureDefaults": false,
			+     "futureDefaults": true,
		`)
};
