const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		"demo/js/main": "./index.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: ({ chunk }) => `${chunk.name.replace("/js/", "/css/")}.css`
		})
	]
};
