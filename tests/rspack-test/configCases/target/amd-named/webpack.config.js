const webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: "NamedLibrary",
		libraryTarget: "amd"
	},
	node: {
		__dirname: false,
		__filename: false
	},
	plugins: [
		new webpack.BannerPlugin({
			raw: true,
			banner: "function define(name, deps, fn) { fn(); }\n"
		})
	]
};
