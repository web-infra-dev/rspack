const path = require("path");
const { default: HtmlPlugin } = require("@rspack/plugin-html");

const prod = process.env.NODE_ENV === "production";

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: { main: "./src/index.tsx" },
	devServer: {
		port: 5555,
		webSocketServer: "sockjs",
		historyApiFallback: true
	},
	mode: prod ? "production" : "development",
	devtool: prod ? false : "source-map",
	builtins: {
		progress: {},
		treeShaking: true,
		sideEffects: true,
		noEmitAssets: false
	},
	cache: false,
	module: {
		rules: [
			{
				test: /\.less$/,
				use: "less-loader",
				type: "css"
			},
			{
				test: /\.module\.less$/,
				use: "less-loader",
				type: "css/module"
			},
			{
				test: /\.svg$/,
				use: "@svgr/webpack"
			},
			{
				test: /\.png$/,
				type: "asset"
			}
		]
	},
	resolve: { alias: { "@": path.resolve(__dirname, "src") } },
	output: {
		publicPath: "/",
		filename: "[name].[contenthash].js"
	},
	optimization: {
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
		new HtmlPlugin({
			title: "Arco Pro App",
			template: path.join(__dirname, "index.html"),
			favicon: path.join(__dirname, "public", "favicon.ico")
		})
	],
	infrastructureLogging: {
		debug: false
	}
};
module.exports = config;
