/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "cache true",
	options: () => ({ cache: true }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": true,
	`)
};
