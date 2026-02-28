const { CopyRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	output: {
		module: true,
		library: {
			type: "module"
		}
	},
	plugins: [
		new CopyRspackPlugin({
			patterns: [{ from: "test.mjs" }]
		})
	]
};
