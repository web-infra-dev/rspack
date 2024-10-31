const { CopyRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		library: {
			type: "umd",
		}
	},
	externals: {
		lodash: {
			root: "_",
			commonjs: "./lodash.js",
			commonjs2: "./lodash.js",
			amd: "./lodash.js"
		}
	},
	externalsType: "umd",
	plugins: [
		new CopyRspackPlugin({
			patterns: ["./lodash.js"]
		})
	]
};
