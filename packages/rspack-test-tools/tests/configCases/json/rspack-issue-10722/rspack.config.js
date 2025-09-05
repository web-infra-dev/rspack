const { CopyRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	output: {
		library: {
			type: "module"
		}
	},
	experiments: {
		outputModule: true
	},
	plugins: [
		new CopyRspackPlugin({
			patterns: [{ from: "test.mjs" }]
		})
	]
};
