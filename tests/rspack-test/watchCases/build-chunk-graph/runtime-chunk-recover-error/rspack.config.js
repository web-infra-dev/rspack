module.exports = {
	entry: {
		entry1: "./entry1.js",
		entry2: "./entry2.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: "single"
	},
	experiments: {
		incremental: {
			buildChunkGraph: true
		}
	}
};
