/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "portable-explicit-root",
	options: () => ({ cache: { type: "persistent", portable: { root: "." } } }),
	diff: e => e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		+   "cache": Object {
		+     "buildDependencies": Array [],
		+     "maxAge": 604800,
		+     "maxMemoryGenerations": Infinity,
		+     "name": "none",
		+     "portable": Object {
		+       "root": "<cwd>",
		+     },
		+     "readonly": false,
		+     "storage": Object {
		+       "directory": "<cwd>/node_modules/.cache/rspack",
		+       "location": "<cwd>/node_modules/.cache/rspack/none",
		+       "type": "filesystem",
		+     },
		+     "type": "persistent",
		+     "version": "",
		+   },
	`)
};
