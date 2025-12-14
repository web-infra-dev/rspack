module.exports = {
	entry: {
		entry1: "./entry1.js",
		entry2: "./entry2.js"
	},
	output: {
		filename: "[name].0.js"
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
