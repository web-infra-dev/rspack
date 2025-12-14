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
				"dynamic-remote": "dynamic_remote@http://localhost:8001/remoteEntry.js"
			},
			shared: {
				'xreact': {},
				'@scope-sc/dep1':{
					singleton:true,
					requiredVersion: '^1.0.0'
				},
				'@scope-sc2/dep2':{
					singleton:false,
					requiredVersion: '>=1.0.0'
				}
			}
		})
	]
};
