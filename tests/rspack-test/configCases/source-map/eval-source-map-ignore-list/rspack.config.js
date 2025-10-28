"use strict";

const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: false,
	plugins: [
		new webpack.EvalSourceMapDevToolPlugin({
			ignoreList: [/ignored\.js/]
		})
	],
	optimization: {
		// Ensure the correct `sourceMappingURL` is detected
		concatenateModules: true
	}
};
