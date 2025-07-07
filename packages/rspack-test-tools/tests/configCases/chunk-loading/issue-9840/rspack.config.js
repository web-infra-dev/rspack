/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		moduleIds: "named"
	},
	experiments: {
		// parallelCodeSplitting: false
	},
	target: "web"
};
