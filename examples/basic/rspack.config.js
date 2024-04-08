const path = require("path");

/**
 * @type {import('webpack').Configuration | import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: "development",
	devtool: false,
	entry: {
		main: "./src/index.js"
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				resourceQuery: /raw/,
				type: "asset/source"
			}
		]
	},
	resolve: {
		alias: {
			"./answer": path.resolve(__dirname, "./src/answer.js?raw")
		}
	}
};
