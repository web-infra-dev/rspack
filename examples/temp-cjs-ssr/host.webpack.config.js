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

const remoteEntryPath = path.join(
	__dirname,
	"dist/webpack-remote/remoteEntry.js"
);

module.exports = {
	target: "async-node",
	mode: "development",
	context: __dirname,
	entry: {
		main: path.resolve(__dirname, "host.js")
	},
	output: {
		path: path.join(__dirname, "dist/webpack-host"),
		filename: "[name].js",
		chunkFormat: "commonjs",
		publicPath: "auto"
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
