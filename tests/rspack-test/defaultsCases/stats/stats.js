/** @type {import('../../../../packages/rspack-test-tools/dist').TDefaultsCaseConfig} */
module.exports = {
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
