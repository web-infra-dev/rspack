const {
	experiments: { VirtualModulesPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new VirtualModulesPlugin({
			"foo.js": 'export const foo = "foo"',
			"bar.js": 'export const bar = "bar"'
		})
	]
};
