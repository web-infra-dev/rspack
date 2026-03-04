const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "named",
		moduleIds: "named"
	},
	output: {
		chunkFilename: "[id].js"
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "ecmascript"
								}
							},
							rspackExperiments: {
								reactServerComponents: true
							}
						}
					}
				]
			},
			{
				test: /exposed-client\.js$/,
				layer: "react-server-components"
			},
			{
				test: /rsc-consumer\.js$/,
				layer: "react-server-components"
			},
			{
				test: /node_modules[\\/]shared-rsc[\\/]index\.js$/,
				layer: "react-server-components"
			},
			{
				issuerLayer: "react-server-components",
				resolve: {
					conditionNames: ["react-server", "..."]
				}
			}
		]
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container.[chunkhash:8].js",
			library: { type: "commonjs-module" },
			manifest: true,
			exposes: {
				"./button": {
					import: "./exposed-client.js",
					layer: "react-server-components"
				},
				"./consumer": {
					import: "./rsc-consumer.js",
					layer: "react-server-components"
				}
			},
			remoteType: "script",
			remotes: {
				remote: "remote@http://localhost:8000/remoteEntry.js"
			},
			shared: {
				"shared-rsc": {
					shareKey: "rsc-shared-key",
					requiredVersion: "^1.0.0"
				}
			}
		})
	]
};
