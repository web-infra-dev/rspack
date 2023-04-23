module.exports = {
	target: "web",
	output: {
		// TODO should be `[name].web.js`
		chunkFilename: "trusted-types.web.js",
		crossOriginLoading: "anonymous",
		trustedTypes: "customPolicyName"
	},
	// performance: {
	// 	hints: false
	// },
	optimization: {
		minimize: false
	}
};
