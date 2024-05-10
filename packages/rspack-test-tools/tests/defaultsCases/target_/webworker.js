module.exports = {
	description: "target webworker",
	options: () => ({ target: "webworker" }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

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
		+       "worker",
		@@ ... @@
		-   "target": "web",
		+   "target": "webworker",
	`)
};
