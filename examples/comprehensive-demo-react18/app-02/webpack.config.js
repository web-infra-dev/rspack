const HtmlWebpackPlugin = require("html-webpack-plugin");
const { ModuleFederationPlugin } = require("@module-federation/enhanced");
const ReactRefreshWebpackPlugin = require("@pmmmwh/react-refresh-webpack-plugin");
const deps = require("./package.json").dependencies;

const isProd = process.env.NODE_ENV === "production";
const isDevelopment = !isProd;

module.exports = {
	entry: "./src/index",
	cache: false,

	mode: "development",
	devtool: "source-map",

	optimization: {
		chunkIds: "named",
		moduleIds: "named",

		minimize: false
	},

	devServer: {
		port: 3002,
		hot: !isProd,
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

	resolve: {
		extensions: [".jsx", ".js", ".json", ".mjs"]
	},

	module: {
		rules: [
			{
				test: /\.m?js$/,
				type: "javascript/auto",
				resolve: {
					fullySpecified: false
				}
			},
			{
				test: /\.jsx?$/,
				exclude: /node_modules/,
				use: [
					{
						loader: require.resolve("babel-loader"),
						options: {
							presets: [require.resolve("@babel/preset-react")],
							plugins: [
								isDevelopment && require.resolve("react-refresh/babel")
							].filter(Boolean)
						}
					}
				]
			}
		]
	},

	plugins: [
		!isProd && new ReactRefreshWebpackPlugin(),
		new ModuleFederationPlugin({
			name: "app_02",
			filename: "remoteEntry.js",
			experiments: { asyncStartup: true },
			remotes: {
				app_01: `app_01@http://localhost:3001/remoteEntry.js`,
				app_03: `app_03@http://localhost:3003/remoteEntry.js`
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
					singleton: true
				},
				react: {
					singleton: true
				}
			}
		}),
		new HtmlWebpackPlugin({ chunks: ["main"] })
	].filter(Boolean)
};
