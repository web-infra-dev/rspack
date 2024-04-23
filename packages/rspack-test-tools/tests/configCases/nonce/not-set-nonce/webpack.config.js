module.exports = {
	target: "web",
	output: {
		chunkFilename: "chunk-without-nonce.web.js",
		crossOriginLoading: "anonymous",
		trustedTypes: true
	},
	optimization: {
		minimize: false
	}
};
