module.exports = ({ config }) => ({
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: true,
		splitChunks: {
			chunks: "all",
			minSize: 0
		}
	}
});
