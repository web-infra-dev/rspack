module.exports = {
	mode: "production",
	entry: "./index",
	output: {
		filename: "[name].js",
		chunkFilename: "[name].js"
	},
	optimization: {
		splitChunks: {
			chunks: "all"
		}
	}
};