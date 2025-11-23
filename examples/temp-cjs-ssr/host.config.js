const path = require("path");
const { ModuleFederationPlugin } = require("@rspack/core").container;

const remoteEntryPath = path.join(
	__dirname,
	"dist/rspack-remote/remoteEntry.js"
);

module.exports = {
	target: "async-node",
	mode: "development",
	context: __dirname,
	entry: {
		main: path.resolve(__dirname, "host.js")
	},
	output: {
		path: path.join(__dirname, "dist/rspack-host"),
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
	resolve: {}
};
