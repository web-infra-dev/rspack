"use strict";

const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./test.js",
	resolve: {
		alias: {
			"ignored-module": false,
			"./ignored-module": false
		}
	},
	plugins: [new webpack.IgnorePlugin({ resourceRegExp: /(b\.js|b)$/ })],
	optimization: {
		sideEffects: true
	}
};
