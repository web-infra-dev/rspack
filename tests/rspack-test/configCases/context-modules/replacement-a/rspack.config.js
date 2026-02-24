"use strict";

const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.ContextReplacementPlugin(
			/replacement.a$/,
			"new-context",
			true,
			/^replaced$/
		)
	]
};
