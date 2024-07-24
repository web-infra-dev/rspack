const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					CssExtractRspackPlugin.loader,
					{
						loader: "css-loader",
						options: {
							modules: {
								mode: "local",
								localIdentName: "foo__[name]__[local]"
							}
						}
					}
				]
			}
		]
	},
	output: {
		module: true
	},
	experiments: {
		outputModule: true
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
