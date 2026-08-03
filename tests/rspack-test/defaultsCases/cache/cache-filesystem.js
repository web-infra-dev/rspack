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
			+     "maxAge": 604800,
			+     "maxVersions": 3,
			+     "name": "default-none",
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
			+       "directory": "<cwd>/node_modules/.cache/rspack",
			+       "location": "<cwd>/node_modules/.cache/rspack/default-none",
			+       "type": "filesystem",
			+     },
			+     "type": "persistent",
			+     "version": "",
			+   },
		`)
};
