const rspack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		libraryTarget: "amd"
	},
	externals: {
		external: "external"
	},
	plugins: [
		new rspack.BannerPlugin({
			raw: true,
			banner: "function define(deps, fn) { fn(); }\n"
		})
	]
};
