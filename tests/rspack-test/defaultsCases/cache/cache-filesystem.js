/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "cache filesystem",
	options: () => ({ cache: { type: "persistent" } }),
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
		`)
};
