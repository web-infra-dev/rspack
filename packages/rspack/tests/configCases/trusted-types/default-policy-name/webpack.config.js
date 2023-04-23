module.exports = {
	target: "web",
	output: {
		// TODO should be `[name].web.js`
		chunkFilename: "default-policy-name.web.js",
		crossOriginLoading: "anonymous",
		trustedTypes: true
	},
	// performance: {
	// 	hints: false
	// },
	optimization: {
		minimize: false
	}
};
