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
			-   "mode": "none",
			+   "mode": "development",
			@@ ... @@
			-         "localIdentName": "[fullhash]",
			+         "localIdentName": "[id]-[local]",
			@@ ... @@
			-         "localIdentName": "[fullhash]",
			+         "localIdentName": "[id]-[local]",
			@@ ... @@
			-         "exportsDepth": 9007199254740991,
			+         "exportsDepth": 1,
			@@ ... @@
			-     "chunkIds": "natural",
			+     "chunkIds": "named",
			@@ ... @@
			-     "moduleIds": "natural",
			-     "nodeEnv": false,
			+     "moduleIds": "named",
			+     "nodeEnv": "development",
			@@ ... @@
			-           "production",
			+           "development",
			@@ ... @@
			-       "production",
			+       "development",
		`)
};
