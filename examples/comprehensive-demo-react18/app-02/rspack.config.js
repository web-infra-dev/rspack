const rspack = require("../../../packages/rspack/dist/index.js");
const {
	HtmlRspackPlugin,
	container: { ModuleFederationPlugin }
} = rspack;

const buildId = Date.now();
const deps = require("./package.json").dependencies;
const ReactRefreshWebpackPlugin = require("@rspack/plugin-react-refresh");
const isProd = process.env.NODE_ENV === "production";

module.exports = {
	entry: "./src/index",

	mode: "development",
	devtool: "source-map",
	resolve: {
		extensions: [".jsx", ".js", ".json", ".mjs"]
	},
	optimization: {
		chunkIds: "named",
		moduleIds: "named",

		minimize: false
	},
	experiments: {
		css: true,
		mfAsyncStartup: true
	},
	devServer: {
		port: 3002,
		hot: true,
		headers: {
			"Access-Control-Allow-Origin": "*",
			"Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, PATCH, OPTIONS",
			"Access-Control-Allow-Headers":
				"X-Requested-With, content-type, Authorization"
		}
	},
	output: {
		publicPath: "auto",
		uniqueName: "app2"
	},

	module: {
		rules: [
			{
				test: /\.jsx?$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						jsc: {
							parser: {
								syntax: "ecmascript",
								jsx: true
							},
							transform: {
								react: {
									development: !isProd,
									refresh: !isProd
								}
							}
						}
					}
				},
				exclude: /node_modules/
			}
		]
	},

	plugins: [
		new ModuleFederationPlugin({
			name: "app_02",
			filename: "remoteEntry.js",
			remotes: {
				app_01: `app_01@http://localhost:3001/remoteEntry.js?v=${buildId}`,
				app_03: `app_03@http://localhost:3003/remoteEntry.js?v=${buildId}`
			},
			exposes: {
				"./Dialog": "./src/Dialog",
				"./Tabs": "./src/Tabs"
			},
			shared: {
				...deps,
				"@mui/material": {
					singleton: true,
					requiredVersion: false
				},
				"react-router-dom": {
					singleton: true
				},
				"react-dom": {
					singleton: true,
					requiredVersion: false
				},
				react: {
					singleton: true,
					requiredVersion: false
				}
			}
		}),
		new HtmlRspackPlugin({
			template: "./public/index.html",
			chunks: ["main"]
		}),
		new ReactRefreshWebpackPlugin()
	]
};
