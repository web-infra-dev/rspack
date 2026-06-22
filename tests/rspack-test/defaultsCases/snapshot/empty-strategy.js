/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "empty snapshot strategies",
	options: () => ({
		snapshot: {
			dependencies: {},
			contextDependencies: {}
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		Compared values have no visual difference.
	`)
};
