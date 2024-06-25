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
