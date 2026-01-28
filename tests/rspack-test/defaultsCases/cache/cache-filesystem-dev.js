/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "cache filesystem development",
	options: () => ({ mode: "development", cache: { type: "persistent" } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-   "cache": false,
			+   "cache": Object {
			+     "buildDependencies": Array [],
			+     "portable": undefined,
			+     "snapshot": Object {
			+       "immutablePaths": Array [],
			+       "managedPaths": Array [
			+         /[\\\\/]node_modules[\\\\/][^.]/,
			+       ],
			+       "unmanagedPaths": Array [],
			+     },
			+     "storage": Object {
			+       "directory": "<cwd>/node_modules/.cache/rspack",
			+       "type": "filesystem",
			+     },
			+     "type": "persistent",
			+     "version": "",
			+   },
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
			-           "production",
			+           "development",
			@@ ... @@
			-       "production",
			+       "development",
		`)
};
