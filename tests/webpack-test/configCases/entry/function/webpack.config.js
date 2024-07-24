/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry() {
		return {
			a: "./a",
			b: ["./b"]
		};
	},
	output: {
		filename: "[name].js"
	}
};
