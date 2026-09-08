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
			+       "immutablePaths": Array [],
			+       "managedPaths": Array [
			+         /[\\\\/]node_modules[\\\\/][^.]/,
			+       ],
			+       "unmanagedPaths": Array [],
			+     },
			+     "type": "memory",
			+   },
		`)
};
