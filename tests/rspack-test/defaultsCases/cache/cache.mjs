/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "cache true",
	options: () => ({ cache: true }),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-   "cache": false,
			+   "cache": Object {
			+     "type": "memory",
			+   },
		`)
};
