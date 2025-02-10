const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /.txt$/,
				type: "javascript/auto",
				use: [
					{
						loader: rspack.CssExtractRspackPlugin.loader
					},
					{
						loader: "css-loader",
						options: {
							modules: {
								namedExport: true,
								exportLocalsConvention: "camel-case-only"
							}
						}
					},
					{
						loader: path.resolve(__dirname, "./loader.js")
					}
				]
			}
		]
	},
	plugins: [new rspack.CssExtractRspackPlugin()],
	experiments: {
		css: false
	},
	resolve: {
		extensions: ["...", ".txt"]
	}
};
