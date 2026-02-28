"use strict";

const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./test.js",
	resolve: {
		alias: {
			"ignored-module": false,
			"./ignored-module": false
		}
	},
	plugins: [new rspack.IgnorePlugin({ resourceRegExp: /(b\.js|b)$/ })],
	optimization: {
		sideEffects: true
	}
};
