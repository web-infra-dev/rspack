/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "experiments.futureDefaults w/ experiments.css disabled",
	options: () => ({
		experiments: {
			css: false,
			futureDefaults: true
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-     "css": undefined,
			+     "css": false,
			@@ ... @@
			-     "futureDefaults": false,
			+     "futureDefaults": true,
		`)
};
