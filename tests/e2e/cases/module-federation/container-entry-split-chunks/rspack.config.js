const { rspack } = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	entry: "./src/index.jsx",
	mode: "development",
	devtool: false,
	resolve: {
		extensions: ["...", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.(jsx?|tsx?)$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript",
									tsx: true
								},
								transform: {
									react: {
										runtime: "automatic",
										development: true,
										refresh: true
									}
								}
							}
						}
					}
				]
			}
		]
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			minSize: 0 // ensure dev server and hmr client is splitted into vendor chunk
		}
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new ReactRefreshPlugin(),
		function (compiler) {
			new rspack.container.ModuleFederationPluginV1({
				name: "remote",
				filename: "remoteEntry.js",
				exposes: {
					"./Component": "./src/RemoteComponent.jsx"
				},
				remotes: {
					remote: `remote@http://localhost:${compiler.options.devServer.port}/remoteEntry.js`
				},
				shared: {
					react: {},
					"react-dom": {}
				}
			}).apply(compiler);
		}
	],
	devServer: {
		hot: true,
		port: 8080,
		devMiddleware: {
			writeToDisk: true
		}
	}
};
