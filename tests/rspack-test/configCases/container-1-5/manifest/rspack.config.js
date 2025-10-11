const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization:{
		chunkIds: 'named',
		moduleIds: 'named'
	},
	output: {
		chunkFilename: '[id].js'
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container.[chunkhash:8].js",
			library: { type: "commonjs-module" },
			exposes: {
				'expose-a': './module.js'
			},
			remotes: {
				remote: 'remote'
			},
			shared: {
				react: {}
			},
			manifest: {
				additionalData({ manifest }) {
					return {
						...manifest,
						extra: true
					};
				}
			}
		})
	]
};
