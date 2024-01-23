/**@type {import("@rspack/cli").Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	
	optimization: {
		sideEffects: true,
	},
};
module.exports = config;
