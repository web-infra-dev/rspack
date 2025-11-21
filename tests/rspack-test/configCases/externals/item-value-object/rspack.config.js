const { CopyRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		library: {
			type: "commonjs"
		}
	},
	externals: {
		lodash: {
			commonjs: "./lodash.js"
		}
	},
	plugins: [
		new CopyRspackPlugin({
			patterns: ["./lodash.js"]
		})
	]
};
