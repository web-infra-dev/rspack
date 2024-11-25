const { CopyRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		library: {
			type: "umd"
		}
	},
	externalsType: "umd",
	externals: {
		lodash: {
			root: "./lodash.js",
			commonjs: "./lodash.js",
			commonjs2: "./lodash.js",
			amd: "./lodash.js"
		}
	},
	plugins: [
		new CopyRspackPlugin({
			patterns: ["./lodash.js"]
		})
	]
};
