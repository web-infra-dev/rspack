const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "async-node",
	optimization: {
		chunkIds: "deterministic",
		moduleIds: "deterministic"
	},
	output: {
		filename: "[name].js",
		chunkFilename: "[id].js",
		uniqueName: "serial-rsc-manifest-runtime-host"
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
			name: "host",
			filename: "remoteEntry.js",
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
			remotes: {
				"@remote/alias": "../0-rsc-manifest-runtime-remote/remoteEntry.js"
			},
			shared: {
				"shared-rsc": {
					request: "shared-rsc",
					import: "shared-rsc",
					shareKey: "rsc-shared-key",
					shareScope: "rsc",
					requiredVersion: "^1.0.0",
					layer: "react-server-components",
					issuerLayer: "react-server-components",
					singleton: true
				}
			}
		})
	]
};
