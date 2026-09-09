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
			+   "cache": Object {
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
			+     "type": "memory",
			+   },
		`)
};
