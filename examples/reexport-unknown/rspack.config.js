// eslint-disable-next-line node/no-unpublished-require

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	devtool: false,
	entry: {
		main: "./index.js"
	},
	optimization: {
		// mangleExports: false,
		minimize: false,
		moduleIds: "named",
		chunkIds: "named",
		sideEffects: false
	},
	builtins: {
		html: [{}]
	},
	plugins: [
	]
};
