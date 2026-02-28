"use strict";

const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		modules: ["...", path.resolve(__dirname, "new-context/modules")]
	},
	plugins: [
		new rspack.ContextReplacementPlugin(
			/replacement.e$/,
			"new-context",
			true,
			/^replaced$|^\.\/modules\/rep/
		)
	]
};
