const config = (index, parallelCodeSplitting) => ({
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
	experiments: {
		parallelCodeSplitting,
		incremental: {
			buildChunkGraph: true
		}
	}
});

module.exports = [true, false].map((parallelCodeSplitting, index) =>
	config(index, parallelCodeSplitting)
);
