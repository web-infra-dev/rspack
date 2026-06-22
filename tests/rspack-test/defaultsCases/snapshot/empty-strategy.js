/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "empty snapshot strategies",
	options: () => ({
		snapshot: {
			module: {},
			contextModule: {}
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		Compared values have no visual difference.
	`)
};
