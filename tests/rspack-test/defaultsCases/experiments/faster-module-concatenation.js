/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "experiments.fasterModuleConcatenation",
	options: () => ({
		experiments: {
			fasterModuleConcatenation: true
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "fasterModuleConcatenation": false,
			+     "fasterModuleConcatenation": true,
		`)
};
