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
			exposes: {
				'expose-a': './module.js'
			},
			remoteType:'script',
			remotes: {
				remote: 'remote@http://localhost:8000/remoteEntry.js'
			},
			shared: {
				react: {}
			},
			manifest: {
				fileName: "custom-manifest.json",
				filePath:'custom-path'
			}
		})
	]
};
