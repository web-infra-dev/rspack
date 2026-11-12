/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
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
