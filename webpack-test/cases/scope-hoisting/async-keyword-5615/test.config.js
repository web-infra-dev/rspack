/**@type {import("@rspack/cli").Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	optimization: {
		concatenateModules: true,
	},
};
module.exports = config;
