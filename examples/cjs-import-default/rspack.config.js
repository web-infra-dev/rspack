/**@type {import("@rspack/cli").Configuration} */
const config = {
	mode: 'production',
	entry: {
		"main": "./index.mjs"
	},
	optimization: {
		mangleExports: false,
		sideEffects: false,
		minimize: false,
		moduleIds: 'named'
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	externals: {
		"worker-threads": false
	},
	target: 'node',
	builtins: {
		treeShaking: false,
	},
};
module.exports = config;
