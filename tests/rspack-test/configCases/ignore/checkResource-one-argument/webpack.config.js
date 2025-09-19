"use strict";

const IgnorePlugin = require("@rspack/core").IgnorePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./test.js",
	plugins: [
		new IgnorePlugin({
			checkResource(resource) {
				return /ignored-module/.test(resource);
			}
		})
	]
};
