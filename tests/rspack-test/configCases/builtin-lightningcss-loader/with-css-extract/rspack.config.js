const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					rspack.CssExtractRspackPlugin.loader,
					"css-loader",
					{
						loader: "builtin:lightningcss-loader",
						/** @type {import("@rspack/core").LightningcssLoaderOptions} */
						options: {
							targets: ["Edge >= 12"]
						}
					}
				],
				type: "javascript/auto"
			}
		]
	},
	plugins: [
		new rspack.CssExtractRspackPlugin({
			filename: "bundle0.css"
		})
	],

};
