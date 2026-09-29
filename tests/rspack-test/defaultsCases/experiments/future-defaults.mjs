/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
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
