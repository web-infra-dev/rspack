const path = require("path");
defineDefaultsCase(Utils.casename(__filename), {
	description: "non-root directory",
	options: () => ({
		cache: {
			type: "filesystem"
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
		+     "type": "filesystem",
		+   },
	`)
});
