const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "amd", name: "NamedLibrary" }
	},
	node: {
		__dirname: false,
		__filename: false
	},
	plugins: [
		new rspack.BannerPlugin({
			raw: true,
			banner: "function define(name, deps, fn) { fn(); }\n"
		})
	]
};
