"use strict";

const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	devtool: false,
	plugins: [
		new webpack.SourceMapDevToolPlugin({
			filename: "[file].map",
			ignoreList: [/ignored\.js/]
		})
	]
};
