/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	target: "node",
	entry: {
		main: "./src/index.js",
		another: "./src/another.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			minSize: {
				js: 1000,
				css: 1000
			}
		}
	}
};
