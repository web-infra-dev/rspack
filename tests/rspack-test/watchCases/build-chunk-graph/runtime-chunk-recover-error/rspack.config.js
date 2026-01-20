const config = index => ({
	entry: {
		entry1: "./entry1.js",
		entry2: "./entry2.js"
	},
	output: {
		filename: `[name].${index}.js`
	},
	optimization: {
		runtimeChunk: "single"
	},
	incremental: {
		buildChunkGraph: true
	}
});

module.exports = [0, 1].map(index => config(index));
