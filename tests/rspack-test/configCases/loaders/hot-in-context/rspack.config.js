const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		// no hmr
	},
	{
		// with hmr
		plugins: [new rspack.HotModuleReplacementPlugin()]
	}
];
