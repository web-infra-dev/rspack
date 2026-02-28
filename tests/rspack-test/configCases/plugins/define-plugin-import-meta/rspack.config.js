var DefinePlugin = require("@rspack/core").DefinePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new DefinePlugin({
			"import.meta.env.MODE": '"production"'
		})
	]
};
