const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	devServer: {
		devMiddleware: {
			writeToDisk: true
		}
	},
	module: {
		rules: [
			{
				resourceQuery: /url/,
				type: "asset"
			},
			{
				test: /complex\.worklet/,
				use: [
					{
						loader: "./loader/worklet-loader.js"
					}
				],
				type: "asset"
			}
		]
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
