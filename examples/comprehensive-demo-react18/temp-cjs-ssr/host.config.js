const path = require("path");
const { ModuleFederationPlugin } = require("@rspack/core").container;

const remoteEntryPath = path.join(__dirname, "dist/remote/remoteEntry.js");

module.exports = {
	target: "node",
	mode: "development",
	context: __dirname,
	experiments: {
		asyncStartup: true
	},
	entry: {
		main: path.resolve(__dirname, "host.js")
	},
	output: {
		path: path.join(__dirname, "dist/host"),
		filename: "[name].js",
		chunkFormat: "commonjs"
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "host",
			remoteType: "commonjs-module",
			experiments: { asyncStartup: true },
			remotes: {
				remote: {
					external: remoteEntryPath,
					shareScope: "default"
				}
			},
			shared: {
				react: { singleton: true, eager: false, requiredVersion: false },
				"react-dom": { singleton: true, eager: false, requiredVersion: false }
			}
		})
	],
	resolve: {
		alias: {
			react: require.resolve("react"),
			"react-dom": require.resolve("react-dom"),
			"react-dom/server": require.resolve("react-dom/server")
		}
	}
};
