module.exports = {
	mode: "development",
	entry: {
		main: "./index.js",
		entry2: "./index2.js"
	},
	output: {
		chunkLoading: "import",
		chunkFormat: "module",
		filename: "[name].[contenthash:8].js",
		chunkFilename: "[name].[contenthash:8].chunk.js"
	},
	optimization: {
		runtimeChunk: true,
		minimize: false
	},
	experiments: {
		css: true,
		outputModule: true
	}
};
