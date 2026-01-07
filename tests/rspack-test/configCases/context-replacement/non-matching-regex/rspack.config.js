var webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		// This plugin should only affect contexts matching /components$/
		// and should NOT affect the "assets" context
		new webpack.ContextReplacementPlugin(
			/components$/,
			"./replaced-components",
			true,
			/^replaced$/
		)
	]
};
