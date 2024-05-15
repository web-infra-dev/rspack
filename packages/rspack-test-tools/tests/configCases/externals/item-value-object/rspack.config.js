const { CopyRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	externals: {
		lodash: {
			root: "_",
			commonjs: "./lodash.js",
			amd: "./lodash.js"
		}
	},
	externalsType: "commonjs",
	plugins: [
		new CopyRspackPlugin({
			patterns: ["./lodash.js"]
		})
	]
};
