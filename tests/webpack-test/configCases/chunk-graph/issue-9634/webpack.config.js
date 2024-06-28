/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		b: "./entry-b",
		a: "./entry-a"
	},
	output: {
		filename: "[name].js"
	}
};
