module.exports = {
	target: "web",
	output: {
		chunkFilename: "chunk-with-nonce.web.js",
		crossOriginLoading: "anonymous",
		trustedTypes: true
	},
	optimization: {
		minimize: false
	}
};
