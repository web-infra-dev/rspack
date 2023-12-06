/**@type {import("@rspack/cli").Configuration} */
const config = {
	target: "node",
	entry: {
	main: "./index.js"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	optimization: {
		minimize: false,
		moduleIds: 'named',
		sideEffects: true,
		usedExports: true
	}
};
module.exports = config;
