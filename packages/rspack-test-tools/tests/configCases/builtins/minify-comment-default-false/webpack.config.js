/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a",
		main: "./index"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		minimize: true
	}
};
