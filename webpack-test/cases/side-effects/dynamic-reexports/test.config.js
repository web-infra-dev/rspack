/**@type {import("@rspack/cli").Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	builtins: {
		treeShaking: false
	},
	optimization: {
		sideEffects: true
	}
};
module.exports = config;
