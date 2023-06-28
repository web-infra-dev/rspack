const webpack = require("../../../../");
/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		// no hmr
	},
	{
		// with hmr
		plugins: [new webpack.HotModuleReplacementPlugin()]
	}
];
