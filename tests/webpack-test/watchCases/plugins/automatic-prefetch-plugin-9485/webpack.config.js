const path = require("path");
const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /delayed/,
				use: path.resolve(__dirname, "./delayed")
			}
		]
	},
	plugins: [new webpack.AutomaticPrefetchPlugin()]
};
