/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	output: {
		chunkFilename: "chunk.js"
	},
	optimization: {
		minimize: true,
		providedExports: true,
		usedExports: true,
		sideEffects: true,
		innerGraph: true
	}
};
