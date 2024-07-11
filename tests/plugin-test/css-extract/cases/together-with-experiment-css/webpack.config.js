const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"],
				type: 'javascript'
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin()
	],
	experiments: {
		css: true
	}
};
