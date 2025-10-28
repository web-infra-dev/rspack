/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "stats false",
	options: () => ({ stats: false }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "stats": Object {},
		+   "stats": Object {
		+     "preset": "none",
		+   },
	`)
};
