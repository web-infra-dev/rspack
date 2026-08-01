/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "experiments.fasterModuleConcatenation",
	options: () => ({
		experiments: {
			fasterModuleConcatenation: false
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "fasterModuleConcatenation": true,
			+     "fasterModuleConcatenation": false,
		`)
};
