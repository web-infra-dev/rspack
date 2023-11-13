const rspack = require("@rspack/core");
const path = require("path");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: ["./src/index.jsx"]
	},
	module: {
		rules: [
			{
				test: /.less$/,
				use: ["less-loader"],
				type: "css"
			}
		]
	},
	optimization: {
		minimize: false, // Disabling minification because it takes too long on CI
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	output: {
		path: path.resolve(__dirname, "dist")
	}
};
module.exports = config;
