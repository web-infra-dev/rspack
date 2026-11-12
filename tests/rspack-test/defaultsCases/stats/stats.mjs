/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "stats true",
	options: () => ({ stats: true }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "stats": Object {},
		+   "stats": Object {
		+     "preset": "normal",
		+   },
	`)
};
