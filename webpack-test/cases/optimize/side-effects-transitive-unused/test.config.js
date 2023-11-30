/**@type {import("@rspack/cli").Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	optimization: {
		sideEffects: true,
		innerGraph: true,
		usedExports: true,
		providedExports: true
	},
	builtins: {
		treeShaking: false,
	},
};
module.exports = config;
