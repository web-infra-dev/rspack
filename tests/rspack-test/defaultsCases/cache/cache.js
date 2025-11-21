/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
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
		@@ ... @@
		-     "unsafeCache": false,
		+     "unsafeCache": /[\\\\/]node_modules[\\\\/]/,
	`)
};
