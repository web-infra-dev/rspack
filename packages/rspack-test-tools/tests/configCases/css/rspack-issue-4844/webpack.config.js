/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js",
		css: "./css"
	},
	output: {
		filename: "[name].js"
	},
	module: {
		generator: {
			"css/auto": {
				exportsConvention: "camel-case",
				exportsOnly: false,
			}
		}
	},
	experiments: {
		css: true
	},
};
