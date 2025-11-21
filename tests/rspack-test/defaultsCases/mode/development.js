/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "development",
	options: () => ({ mode: "development" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": true,
		@@ ... @@
		-   "devtool": false,
		+   "devtool": "eval",
		@@ ... @@
		-     "cache": false,
		+     "cache": true,
		@@ ... @@
		-   "mode": "none",
		+   "mode": "development",
		@@ ... @@
		-         "exportsDepth": 9007199254740991,
		+         "exportsDepth": 1,
		@@ ... @@
		-     "unsafeCache": false,
		+     "unsafeCache": /[\\\\/]node_modules[\\\\/]/,
		@@ ... @@
		-     "chunkIds": "natural",
		+     "chunkIds": "named",
		@@ ... @@
		-     "moduleIds": "natural",
		-     "nodeEnv": false,
		+     "moduleIds": "named",
		+     "nodeEnv": "development",
		@@ ... @@
		-     "pathinfo": false,
		+     "pathinfo": true,
		@@ ... @@
		-       "production",
		+       "development",
	`)
};
