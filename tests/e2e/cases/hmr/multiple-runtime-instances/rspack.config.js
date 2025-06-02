const rspack = require('@rspack/core');

/** @type import('@rspack/core').Configuration */
module.exports ={
	mode: "development",
	entry: "./src/index.js",
	output: {
		filename: "bundle.js",
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html",
			// Skip default script injection
			chunks: []
		})
	],
};
