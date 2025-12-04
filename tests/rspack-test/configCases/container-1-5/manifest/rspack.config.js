const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization:{
		chunkIds: 'named',
		moduleIds: 'named'
	},
	output: {
		chunkFilename: '[id].js',
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container.[chunkhash:8].js",
			library: { type: "commonjs-module" },
			manifest: true,
			exposes: {
				'./expose-a': {
					import: './module.js',
					name:'_federation_expose_a'
				}
			},
			remoteType:'script',
			remotes: {
				'@remote/alias': 'remote@http://localhost:8000/remoteEntry.js',
				'@dynamic-remote/alias': 'dynamic_remote@http://localhost:8001/remoteEntry.js',
				'@scope-scope/ui': 'ui@http://localhost:8002/remoteEntry.js'
			},
			shared: {
				react: {}
			}
		})
	]
};
