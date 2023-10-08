const stats = {
	hash: false,
	timings: false,
	builtAt: false,
	assets: false,
	chunks: true,
	chunkRelations: true,
	entrypoints: true,
	modules: false
};
/** @type {import("../../../").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		main: "./"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			minSize: 100,
		}
	},
	stats
};
