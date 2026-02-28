/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		main: "./index.js"
	},
	output: {
		filename: "[name].js",
		library: { type: "umd", name: "MyLibrary" },
		chunkLoading: "jsonp",
		chunkFormat: "array-push",
		globalObject: "this"
	},
	optimization: {
		minimize: false,
		runtimeChunk: "single"
	}
};
