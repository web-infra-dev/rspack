const rspack = require("../../../packages/rspack/dist/index.js");
const {
	HtmlRspackPlugin,
	container: { ModuleFederationPlugin }
} = rspack;
const buildId = Date.now();

const isProd = process.env.NODE_ENV === "production";

module.exports = {
	entry: "./src/index",

	mode: "development",
	devtool: "source-map",
	optimization: {
		chunkIds: "named",
		moduleIds: "named",

		minimize: false
	},
	devServer: {
		port: 3003,
		hot: true,
		headers: {
			"Access-Control-Allow-Origin": "*",
			"Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, PATCH, OPTIONS",
			"Access-Control-Allow-Headers":
				"X-Requested-With, content-type, Authorization"
		}
	},
	resolve: {
		extensions: [".jsx", ".js", ".json", ".mjs"]
	},
	output: {
		publicPath: "auto",
		uniqueName: "app3"
	},
	experiments: {
		css: true
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
		new HtmlRspackPlugin({
			templateContent: () =>
				`<!doctype html>\n<html lang="en">\n<head>\n  <meta charset="utf-8" />\n  <title>App 03</title>\n</head>\n<body>\n  <div id="root"></div>\n</body>\n</html>`
		}),
		new ModuleFederationPlugin({
			name: "app_03",
			filename: "remoteEntry.js",
			remotes: {
				app_01: `app_01@http://localhost:3001/remoteEntry.js?v=${buildId}`
			},
			exposes: {
				"./Button": "./src/Button"
			},
			shared: {
				"react-dom": {
					singleton: true
				},
				react: {
					singleton: true
				}
			},
			experiments: {
				asyncStartup: true
			}
		})
	]
};
