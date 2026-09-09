const path = require("path");
/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "non-root directory",
	options: () => ({
		cache: {
			type: "persistent"
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
			+     "buildDependencies": Array [],
			+     "maxAge": 604800,
			+     "maxMemoryGenerations": Infinity,
			+     "name": "none",
			+     "portable": false,
			+     "readonly": false,
			+     "snapshot": Object {
			+       "buildDependencies": Object {
			+         "hash": true,
			+         "timestamp": true,
			+       },
			+       "contextModule": Object {
			+         "timestamp": true,
			+       },
			+       "immutablePaths": Array [],
			+       "managedPaths": Array [
			+         /[\\\\/]node_modules[\\\\/][^.]/,
			+       ],
			+       "module": Object {
			+         "timestamp": true,
			+       },
			+       "resolve": Object {
			+         "timestamp": true,
			+       },
			+       "resolveBuildDependencies": Object {
			+         "hash": true,
			+         "timestamp": true,
			+       },
			+       "unmanagedPaths": Array [],
			+     },
			+     "storage": Object {
			+       "directory": "<cwd>/fixtures/node_modules/.cache/rspack",
			+       "location": "<cwd>/fixtures/node_modules/.cache/rspack/none",
			+       "type": "filesystem",
			+     },
			+     "type": "persistent",
			+     "version": "",
			+   },
		`)
};
