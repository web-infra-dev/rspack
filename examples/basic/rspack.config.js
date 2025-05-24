const { ModuleFederationPlugin } = require("@rspack/core").container;

function createConfig() {
	return {
		target: "node",
		entry: "./index.js",
		mode: "development",
		output: {
			filename: "[name].js",
			publicPath: "/"
		},
		optimization: {
			minimize: false,
			chunkIds: "named"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container",
				remoteType: "commonjs-module",
				library: { type: "commonjs-module" },
				exposes: ["./a"],
				remotes: {
					container2:
						"promise Promise.resolve().then(() => require('./container2.js'))"
				}
			}),
			new ModuleFederationPlugin({
				name: "container2",
				remoteType: "commonjs-module",
				library: { type: "commonjs-module" },
				exposes: ["./b"],
				remotes: {
					container:
						"promise Promise.resolve().then(() => require('./container.js'))"
				}
			})
		]
	};
}

module.exports = createConfig();
