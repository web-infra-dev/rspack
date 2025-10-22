const path = require("path");
/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "non-root directory",
	options: () => ({
		cache: {
			type: "filesystem"
		}
	}),
	cwd: path.resolve(__dirname, "../../fixtures"),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": Object {
		+     "type": "filesystem",
		+   },
		@@ ... @@
		-     "unsafeCache": false,
		+     "unsafeCache": /[\\\\/]node_modules[\\\\/]/,
	`)
};
