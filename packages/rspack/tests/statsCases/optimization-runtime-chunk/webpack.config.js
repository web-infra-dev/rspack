/** @type {import('@rspack/core').Configuration} */
module.exports = {
	mode: "development",
	entry: {
		e1: "./e1",
		e2: {
			import: "./e2",
			runtime: "e2~runtime"
		}
	},
	output: {
		filename: "[name].js",
		chunkFilename: "[name].chunk.js"
	},
	stats: {
		all: false,
		reasons: true,
		chunks: true,
		entrypoints: true,
		chunkGroups: true,
		errors: true,
	},
	optimization: {
		runtimeChunk: {
			name: (entrypoint) => `${entrypoint.name}~runtime`
		}
	}
};
