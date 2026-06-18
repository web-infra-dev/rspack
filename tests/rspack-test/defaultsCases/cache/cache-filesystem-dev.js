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
			+     "portable": false,
			+     "readonly": false,
			+     "snapshot": Object {
			+       "immutablePaths": Array [],
			+       "managedPaths": Array [
			+         /[\\\\/]node_modules[\\\\/][^.]/,
			+       ],
			+       "unmanagedPaths": Array [],
			+     },
			+     "storage": Object {
			+       "directory": "<cwd>/node_modules/.cache/rspack/development",
			+       "maxAge": undefined,
			+       "maxGenerations": undefined,
			+       "type": "filesystem",
			+     },
			+     "type": "persistent",
			+     "version": "",
			+   },
			@@ ... @@
			-   "devtool": false,
			+   "devtool": "cheap-module-source-map",
			@@ ... @@
			-   "mode": "none",
			+   "mode": "development",
			@@ ... @@
			-         "localIdentName": "[fullhash]",
			+         "localIdentName": "[uniqueName]-[id]-[local]",
			@@ ... @@
			-         "localIdentName": "[fullhash]",
			+         "localIdentName": "[uniqueName]-[id]-[local]",
			@@ ... @@
			-         "localIdentName": "[fullhash]",
			+         "localIdentName": "[uniqueName]-[id]-[local]",
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
			-           "production",
			+           "development",
			@@ ... @@
			-           "production",
			+           "development",
			@@ ... @@
			-       "production",
			+       "development",
		`)
};
