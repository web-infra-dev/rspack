module.exports = {
	target: "node",
	optimization: {
		chunkIds: "named"
	},
	output: {
		chunkFilename: "[name].js"
	},
	node: {
		__dirname: false
	}
};
