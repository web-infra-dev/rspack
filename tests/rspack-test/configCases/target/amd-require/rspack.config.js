const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "amd-require" }
	},
	node: {
		__dirname: false,
		__filename: false
	},
	plugins: [
		new rspack.BannerPlugin({
			raw: true,
			banner:
				"var nodeRequire = require;\nvar require = function(deps, fn) { fn(); }\n"
		})
	]
};
