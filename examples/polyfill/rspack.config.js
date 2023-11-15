const rspack = require("@rspack/core");
const path = require("path");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	resolve: {
		alias: {
			"core-js": path.dirname(require.resolve("core-js"))
		}
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				exclude: /node_modules/,
				loader: "builtin:swc-loader",
				options: {
					sourceMap: true,
					env: {
						targets: ["> 0.01%", "not dead", "not op_mini all"],
						mode: "usage",
						coreJs: "3.26"
					}
				}
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
