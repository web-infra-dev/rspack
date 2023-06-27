"use strict";

const IgnorePlugin = require("../../../../").IgnorePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./test.js",
	plugins: [
		new IgnorePlugin({
			resourceRegExp: /ignored-module/,
			contextRegExp: /folder-b/
		})
	]
};
