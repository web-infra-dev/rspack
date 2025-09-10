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
			"translations/en.js": 'export const hello = "hello"',
			"translations/zh.js": 'export const hello = "你好"'
		})
	]
};
