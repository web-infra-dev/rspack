const path = require("path");
const webpack = require("webpack");
const ReactRefreshPlugin = require("@pmmmwh/react-refresh-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");

const prod = process.env.NODE_ENV === "production";

/** @type {import('webpack').Configuration} */
const config = {
	mode: 'development',
	context: __dirname,
	entry: "./src/index.tsx",
	target: ["web", "es5"],
	devServer: {
		port: 5555,
		webSocketServer: "sockjs",
		historyApiFallback: true
	},
	module: {
		rules: [
			{
				test: /\.less$/,
				use: ["style-loader", "css-loader", "less-loader"],
				exclude: /\.module\.less$/
			},
			{
				test: /\.module\.less$/,
				use: ["style-loader", {
					loader: "css-loader",
					options: {
						modules: true,
						importLoaders: 1,
					},
				}, "less-loader"],
			},
			{
				test: /\.svg$/,
				use: "@svgr/webpack"
			},
			{
				test: /\.(j|t)s$/,
				exclude: [/[\\/]node_modules[\\/]/],
				loader: "swc-loader",
				options: {
					sourceMap: false,
					jsc: {
						parser: {
							syntax: "typescript"
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			},
			{
				test: /\.(j|t)sx$/,
				loader: "swc-loader",
				exclude: [/[\\/]node_modules[\\/]/],
				options: {
					sourceMap: false,
					jsc: {
						parser: {
							syntax: "typescript",
							tsx: true
						},
						transform: {
							react: {
								runtime: "automatic",
								development: !prod,
								refresh: false
							}
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			},
			{
				test: /\.png$/,
				type: "asset"
			}
		]
	},
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "src"),
			// The default exported mock.js contains a minified [parser](https://github.com/nuysoft/Mock/blob/refactoring/src/mock/regexp/parser.js) with super deep binary
			// expression, which causes stack overflow for swc parser in debug mode.
			// Alias to the unminified version mitigates this problem.
			// See also <https://github.com/search?q=repo%3Aswc-project%2Fswc+parser+stack+overflow&type=issues>
			mockjs: require.resolve("./patches/mock.js")
		},
		extensions: [".js", ".jsx", ".ts", ".tsx", ".css", ".less"]
	},
	output: {
		publicPath: "/",
		filename: "[name].js",
		chunkFilename: "[name].js"
	},
	optimization: {
		minimize: false, // Disabling minification because it takes too long on CI
		realContentHash: true,
		splitChunks: {
			cacheGroups: {
				someVendor: {
					chunks: "all",
					minChunks: 2
				}
			}
		}
	},
	plugins: [
		new HtmlWebpackPlugin({
			title: "Arco Pro App",
			template: path.join(__dirname, "index.html"),
			favicon: path.join(__dirname, "public", "favicon.ico")
		}),
		// new ReactRefreshPlugin(),
		new webpack.ProgressPlugin()
	],
	infrastructureLogging: {
		debug: false
	},
};
module.exports = config;
