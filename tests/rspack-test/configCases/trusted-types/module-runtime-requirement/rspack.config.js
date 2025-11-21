module.exports = {
	output: {
		filename: "[name].js",
		chunkFilename: "[name].js",
		trustedTypes: true
	},
	node: {
		__dirname: false,
		__filename: false
	},
	devtool: "eval-source-map",
	target: "web"
};
