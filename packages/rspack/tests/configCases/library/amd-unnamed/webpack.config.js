const rspack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		libraryTarget: "amd"
	},
	plugins: [
		new rspack.BannerPlugin({
			raw: true,
			banner: "function define(fn) { fn(); }\n"
		})
	]
};
