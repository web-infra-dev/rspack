module.exports = {
	target: "web",
	optimization: {
		splitChunks: {
			minSize: 0,
			chunks: "all",
		}
	}
};
