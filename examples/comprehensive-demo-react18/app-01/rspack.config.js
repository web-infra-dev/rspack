const rspack = require("../../../packages/rspack/dist/index.js");
const {
	HtmlRspackPlugin,
	container: { ModuleFederationPlugin }
} = rspack;

const { RsdoctorRspackPlugin } = require("@rsdoctor/rspack-plugin");
const ReactRefreshWebpackPlugin = require("@rspack/plugin-react-refresh");

const deps = require("./package.json").dependencies;
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
	output: {
		publicPath: "auto",
		uniqueName: "app1"
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
			},
			{
				test: /\.md$/,
				type: "asset/source"
			}
		]
	},
	devServer: {
		port: 3001,
		hot: true,
		headers: {
			"Access-Control-Allow-Origin": "*",
			"Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, PATCH, OPTIONS",
			"Access-Control-Allow-Headers":
				"X-Requested-With, content-type, Authorization"
		}
	},
	plugins: [
		new HtmlRspackPlugin({
			templateContent: () =>
				`<!doctype html>\n<html lang="en">\n<head>\n  <meta charset="utf-8" />\n  <title>App 01</title>\n</head>\n<body>\n  <div id="root"></div>\n</body>\n</html>`
		}),
		new ModuleFederationPlugin({
			name: "app_01",
			filename: "remoteEntry.js",
			remotes: {
				app_02: `app_02@http://localhost:3002/remoteEntry.js`,
				app_03: `app_03@http://localhost:3003/remoteEntry.js`,
				app_04: `app_04@http://localhost:3004/remoteEntry.js`
			},
			exposes: {
				"./SideNav": "./src/SideNav",
				"./Page": "./src/Page"
			},
			shared: {
				...deps,
				"@mui/material": {
					singleton: true,
					requiredVersion: false
				},
				"react-router-dom": {
					singleton: true,
					requiredVersion: false
				},
				"react-dom": {
					singleton: true,
					requiredVersion: false
				},
				react: {
					singleton: true,
					requiredVersion: false
				}
			},
			experiments: {
				asyncStartup: true
			}
		}),
		isProd ? new ReactRefreshWebpackPlugin() : undefined
		// new RsdoctorRspackPlugin()
	]
};
