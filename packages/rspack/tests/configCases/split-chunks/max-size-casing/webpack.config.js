/** @type {import("../../../../").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	experiments: {
		newSplitChunks: true
	},
	optimization: {
		chunkIds: "named",
		splitChunks: {
			hidePathInfo: false,
			minSize: 50,
			maxSize: 100
		}
	}
};
