/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: "./index.js",
			runtime: false
		}
	},
	output: {
		filename: "[name].js"
	}
};
