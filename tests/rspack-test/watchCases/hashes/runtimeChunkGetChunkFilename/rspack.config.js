module.exports = {
	mode: "development",
	entry: {
		main: "./index.js",
		entry2: "./index2.js"
	},
	output: {
		module: true,
		chunkLoading: "import",
		chunkFormat: "module",
		filename: "[name].[contenthash:8].js",
		chunkFilename: "[name].[contenthash:8].chunk.js"
	},
	optimization: {
		runtimeChunk: true,
		minimize: false
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	}
};
