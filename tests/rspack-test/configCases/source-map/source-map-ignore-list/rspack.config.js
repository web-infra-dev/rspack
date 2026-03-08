"use strict";

const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	devtool: false,
	plugins: [
		new rspack.SourceMapDevToolPlugin({
			filename: "[file].map",
			ignoreList: [/ignored\.js/]
		})
	]
};
