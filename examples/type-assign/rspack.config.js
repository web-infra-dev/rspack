/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			name: "MyLibrary",
			type: "assign",
		},
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},

	optimization: {
		sideEffects: true,
		innerGraph: true,
		providedExports: true,
		usedExports: true,
		moduleIds: 'named',
		minimize: false
	},
	builtins: {
		treeShaking: false,
	},
	entry: "./index.js",
};
