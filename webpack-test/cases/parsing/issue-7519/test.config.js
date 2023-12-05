/**@type {import("@rspack/cli").Configuration} */
console.log(process.env.NODE_ENV)
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true, // related to dead branch remover
		},
	},
	builtins: {
		treeShaking: false,
	},
};
module.exports = config;
