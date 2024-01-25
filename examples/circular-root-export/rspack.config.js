const rspack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
const config = {
	mode: "production",
	entry: {
		main: "./index.js",
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	// plugins: [new rspack.HtmlRspackPlugin({ template: "./index.html" })],
	optimization: {
		concatenateModules: true,
		minimize: false,
		moduleIds: "named",
		chunkIds: 'named',
		mangleExports: false,
	},
};

module.exports = config;
