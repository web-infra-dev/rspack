/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "output module",
	options: () => ({ experiments: { outputModule: true } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		+     "outputModule": true,
		@@ ... @@
		-   "externalsType": "var",
		+   "externalsType": "module-import",
		@@ ... @@
		-       "dynamicImport": undefined,
		-       "dynamicImportInWorker": undefined,
		+       "dynamicImport": true,
		+       "dynamicImportInWorker": true,
		@@ ... @@
		-       "module": undefined,
		+       "module": true,
		@@ ... @@
		-     "chunkFilename": "[name].js",
		-     "chunkFormat": "array-push",
		+     "chunkFilename": "[name].mjs",
		+     "chunkFormat": "module",
		@@ ... @@
		-     "chunkLoading": "jsonp",
		+     "chunkLoading": "import",
		@@ ... @@
		-       "jsonp",
		-       "import-scripts",
		+       "import",
		@@ ... @@
		-       "dynamicImport": undefined,
		-       "dynamicImportInWorker": undefined,
		+       "dynamicImport": true,
		+       "dynamicImportInWorker": true,
		@@ ... @@
		-       "module": undefined,
		+       "module": true,
		@@ ... @@
		-     "filename": "[name].js",
		+     "filename": "[name].mjs",
		@@ ... @@
		-     "hotUpdateChunkFilename": "[id].[fullhash].hot-update.js",
		+     "hotUpdateChunkFilename": "[id].[fullhash].hot-update.mjs",
		@@ ... @@
		-     "hotUpdateMainFilename": "[runtime].[fullhash].hot-update.json",
		-     "iife": true,
		+     "hotUpdateMainFilename": "[runtime].[fullhash].hot-update.json.mjs",
		+     "iife": false,
		@@ ... @@
		-     "module": false,
		+     "module": true,
		@@ ... @@
		-     "scriptType": false,
		+     "scriptType": "module",
		@@ ... @@
		-     "workerChunkLoading": "import-scripts",
		+     "workerChunkLoading": "import",
	`)
};
