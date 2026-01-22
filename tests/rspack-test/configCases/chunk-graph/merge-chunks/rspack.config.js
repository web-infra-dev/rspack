const rspack = require("@rspack/core");

/** @type {import(@rspack/core).Configuration} */
module.exports = {
	entry: {
		entry1: "./entry1.js",
		entry2: "./entry2.js"
	},
	output: {
		filename: "[name].js"
	},
	module: {
		rules: [
			{
				test: /\.css/,
				type: "javascript/auto",
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [new rspack.CssExtractRspackPlugin()],
	experiments: { css: false }
};
