const HtmlWebpackPlugin = require("html-webpack-plugin");
const { ModuleFederationPlugin } = require("@module-federation/enhanced");
const ReactRefreshWebpackPlugin = require("@pmmmwh/react-refresh-webpack-plugin");

const isProd = process.env.NODE_ENV === "production";

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

	output: {
		publicPath: "auto",
		uniqueName: "app3"
	},

	resolve: {
		extensions: [".jsx", ".js", ".json", ".mjs"]
	},
	devServer: {
		port: 3003,
		hot: !isProd,
		headers: {
			"Access-Control-Allow-Origin": "*",
			"Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, PATCH, OPTIONS",
			"Access-Control-Allow-Headers":
				"X-Requested-With, content-type, Authorization"
		}
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
				loader: require.resolve("babel-loader"),
				exclude: /node_modules/,
				options: {
					presets: [require.resolve("@babel/preset-react")],
					plugins: [!isProd && require.resolve("react-refresh/babel")].filter(
						Boolean
					)
				}
			}
		]
	},

	plugins: [
		!isProd && new ReactRefreshWebpackPlugin(),
		new ModuleFederationPlugin({
			name: "app_03",
			filename: "remoteEntry.js",
			experiments: { asyncStartup: true },
			remotes: {
				app_01: `app_01@http://localhost:3001/remoteEntry.js`
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
			}
		}),
		new HtmlWebpackPlugin({
			template: "./public/index.html"
		})
	].filter(Boolean)
};
