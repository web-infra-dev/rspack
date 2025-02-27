const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./src/index.js",
	context: __dirname,
	mode: "development",
	plugins: [new rspack.HtmlRspackPlugin()],
	module: {
		rules: [
			{
				test: /\.js$/,
				exclude: [/node_modules/],
				include: [/src/],
				loader: "builtin:swc-loader"
			}
		]
	},
	devServer: {
		hot: true
	}
};
