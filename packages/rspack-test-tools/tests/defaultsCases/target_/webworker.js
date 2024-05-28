/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "target webworker",
	options: () => ({ target: "webworker" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-       "document": true,
		+       "document": false,
		@@ ... @@
		-         "exportsOnly": false,
		+         "exportsOnly": true,
		@@ ... @@
		-         "exportsOnly": false,
		+         "exportsOnly": true,
		@@ ... @@
		-         "exportsOnly": false,
		+         "exportsOnly": true,
		@@ ... @@
		-     "chunkLoading": "jsonp",
		+     "chunkLoading": "import-scripts",
		@@ ... @@
		-       "jsonp",
		@@ ... @@
		-       "document": true,
		+       "document": false,
		@@ ... @@
		+       "worker",
		@@ ... @@
		-   "target": "web",
		+   "target": "webworker",
	`)
};
