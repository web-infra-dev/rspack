const path = require("path");
const resolveFrom = request =>
	require.resolve(request, {
		paths: [
			__dirname,
			path.resolve(
				__dirname,
				"../comprehensive-demo-react18/app-02/node_modules"
			),
			path.resolve(__dirname, "../../node_modules")
		]
	});

const { ModuleFederationPlugin } = require(
	resolveFrom("@module-federation/enhanced/webpack")
);

module.exports = {
	target: "async-node",
	mode: "development",
	context: __dirname,
	entry: {
		remote: path.resolve(__dirname, "remote.js")
	},
	output: {
		path: path.join(__dirname, "dist/webpack-remote"),
		filename: "[name].js",
		chunkFormat: "commonjs",
		library: { type: "commonjs-module" },
		publicPath: "auto"
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
