const HtmlWebpackPlugin = require("html-webpack-plugin");
const { ModuleFederationPlugin } = require("@module-federation/enhanced");
const { RsdoctorWebpackPlugin } = require("@rsdoctor/webpack-plugin");
const ReactRefreshWebpackPlugin = require("@pmmmwh/react-refresh-webpack-plugin");

const isProd = process.env.NODE_ENV === "production";
const isDevelopment = !isProd;

const deps = require("./package.json").dependencies;
module.exports = {
	entry: "./src/index.jsx",
	cache: false,
	devServer: {
		port: 3001,
		hot: isDevelopment,
		headers: {
			"Access-Control-Allow-Origin": "*",
			"Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, PATCH, OPTIONS",
			"Access-Control-Allow-Headers":
				"X-Requested-With, content-type, Authorization"
		}
	},
	mode: "development",
	devtool: "source-map",

	optimization: {
		chunkIds: "named",
		moduleIds: "named",

		minimize: false
	},

	output: {
		uniqueName: "app1",
		publicPath: "auto"
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
			},
			{
				test: /\.md$/,
				loader: "raw-loader"
			}
		]
	},

	plugins: [
		isDevelopment && new ReactRefreshWebpackPlugin(),
		new ModuleFederationPlugin({
			name: "app_01",
			filename: "remoteEntry.js",
			experiments: { asyncStartup: true },
			remotes: {
				app_02: `app_02@http://localhost:3002/remoteEntry.js`,
				app_03: `app_03@http://localhost:3003/remoteEntry.js`,
				app_04: `app_04@http://localhost:3004/remoteEntry.js`,
				app_05: `app_05@http://localhost:3005/remoteEntry.js`
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
			}
		}),
		new HtmlWebpackPlugin({ chunks: ["main"] })
		// new RsdoctorWebpackPlugin({
		//   // plugin options
		// }),
	].filter(Boolean)
};
