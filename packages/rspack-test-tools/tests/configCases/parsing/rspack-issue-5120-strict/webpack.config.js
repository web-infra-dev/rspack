/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js",
		fail: "./fail.js"
	},
	output: {
		filename: "[name].js"
	}
};
