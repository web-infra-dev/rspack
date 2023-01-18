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
		hash: false,
		timings: false,
		builtAt: false,
		assets: false,
		modules: false,
		reasons: true
	},
	optimization: {
		runtimeChunk: {
			name: (entrypoint) => `${entrypoint.name}~runtime`
		}
	}
};
