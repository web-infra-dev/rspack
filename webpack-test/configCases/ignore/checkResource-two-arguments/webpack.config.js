"use strict";

const IgnorePlugin = require("../../../../").IgnorePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./test.js",
	plugins: [
		new IgnorePlugin({
			checkResource(resource, context) {
				return /ignored-module/.test(resource) && /folder-b/.test(context);
			}
		})
	]
};
