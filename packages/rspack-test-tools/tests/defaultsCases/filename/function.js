module.exports = {
	description: "function filename",
	options: () => ({ output: { filename: () => "bundle.js" } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkFilename": "[name].js",
		+     "chunkFilename": "[id].js",
		@@ ... @@
		-     "cssChunkFilename": "[name].css",
		-     "cssFilename": "[name].css",
		+     "cssChunkFilename": "[id].css",
		+     "cssFilename": "[id].css",
		@@ ... @@
		-     "filename": "[name].js",
		+     "filename": [Function filename],
	`)
};
