/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "const filename",
	options: () => ({ output: { filename: "bundle.js" } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkFilename": "[name].js",
		+     "chunkFilename": "[id].bundle.js",
		@@ ... @@
		-     "cssChunkFilename": "[name].css",
		-     "cssFilename": "[name].css",
		+     "cssChunkFilename": "[id].bundle.css",
		+     "cssFilename": "bundle.css",
		@@ ... @@
		-     "filename": "[name].js",
		+     "filename": "bundle.js",
	`)
};
