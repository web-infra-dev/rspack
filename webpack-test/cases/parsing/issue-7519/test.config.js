/**@type {import("@rspack/cli").Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: false, // related to dead branch remover
		},
	},
	builtins: {
		treeShaking: false,
	},
};
module.exports = config;
