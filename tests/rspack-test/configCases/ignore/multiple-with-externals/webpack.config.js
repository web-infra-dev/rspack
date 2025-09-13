"use strict";

const IgnorePlugin = require("@rspack/core").IgnorePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./test.js",
	externals: {
		"./normal-module": "{}"
	},
	plugins: [
		new IgnorePlugin({
			resourceRegExp: /ignored-module1/
		}),
		new IgnorePlugin({
			resourceRegExp: /ignored-module2/
		})
	]
};
