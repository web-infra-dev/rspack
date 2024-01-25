/**@type {import("@rspack/cli").Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	optimization: {
		concatenateModules: true
	},
	builtins: {
		treeShaking: false,
	},
};
module.exports = config;

