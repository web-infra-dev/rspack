const path = require("path");
const { ModuleFederationPlugin } = require("@rspack/core").container;

module.exports = {
	target: "async-node",
	mode: "development",
	context: __dirname,
	entry: {
		remote: path.resolve(__dirname, "remote.js")
	},
	output: {
		path: path.join(__dirname, "dist/rspack-remote"),
		filename: "[name].js",
		chunkFormat: "commonjs",
		library: { type: "commonjs-module" }
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "remote",
			filename: "remoteEntry.js",
			library: { type: "commonjs-module" },
			experiments: { asyncStartup: true },
			exposes: {
				"./Widget": "./remote.js"
			},
			shared: {
				react: { singleton: true, eager: false, requiredVersion: false },
				"react-dom": { singleton: true, eager: false, requiredVersion: false }
			}
		})
	],
	resolve: {}
};
