var webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new webpack.ContextReplacementPlugin(
			/context-replacement.a$/,
			"new-context",
			true,
			/^replaced$/
		)
	]
};
