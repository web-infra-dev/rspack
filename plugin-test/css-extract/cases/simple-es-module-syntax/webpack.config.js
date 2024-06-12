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
						loader: "css-loader"
						// TODO Uncomment after `css-loader` release the `esModule` option
						// options: { esModule: true },
					}
				]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
