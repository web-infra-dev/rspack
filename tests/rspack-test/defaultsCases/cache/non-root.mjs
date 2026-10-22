import path from "node:path";
/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "non-root directory",
	options: () => ({
		cache: {
			type: "persistent"
		}
	}),
	cwd: path.resolve(import.meta.dirname, "../../fixtures"),
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
