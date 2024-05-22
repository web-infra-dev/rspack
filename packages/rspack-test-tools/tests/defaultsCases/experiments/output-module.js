/** @type {import('../../..').TDefaultsCaseConfig} */
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
		+   "externalsType": "module",
		@@ ... @@
		-     "chunkFilename": "[name].js",
		+     "chunkFilename": "[name].mjs",
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
		-     "iife": true,
		+     "iife": false,
		@@ ... @@
		-     "module": false,
		+     "module": true,
		@@ ... @@
		-     "scriptType": false,
		+     "scriptType": "module",
	`)
};
