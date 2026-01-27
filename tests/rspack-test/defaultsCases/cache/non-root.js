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
			+     "portable": undefined,
			+     "snapshot": Object {
			+       "immutablePaths": Array [],
			+       "managedPaths": Array [
			+         /[\\\\/]node_modules[\\\\/][^.]/,
			+       ],
			+       "unmanagedPaths": Array [],
			+     },
			+     "storage": Object {
			+       "directory": "<cwd>/fixtures/node_modules/.cache/rspack",
			+       "type": "filesystem",
			+     },
			+     "type": "persistent",
			+     "version": "",
			+   },
			@@ ... @@
			-     "unsafeCache": false,
			+     "unsafeCache": /[\\\\/]node_modules[\\\\/]/,
		`)
};
