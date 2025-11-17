const rspack = require("../../../packages/rspack/dist/index.js");
const {
	HtmlRspackPlugin,
	container: { ModuleFederationPlugin }
} = rspack;
const buildId = Date.now();
const path = require("path");

const mode = process.env.NODE_ENV || "development";
const prod = mode === "production";

module.exports = {
	entry: {
		bundle: ["./src/main.js"]
	},
	resolve: {
		extensions: [".mjs", ".js", ".svelte"],
		mainFields: ["svelte", "browser", "module", "main"],
		conditionNames: ["svelte", "browser", "import"]
	},
	output: {
		path: path.resolve(__dirname, "public"),
		filename: "[name].[contenthash:8].js",
		chunkFilename: "[name].[contenthash:8].js",
		publicPath: "auto",
		uniqueName: "app4"
	},
	devServer: {
		port: 3004,
		hot: true,
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
				test: /\.svelte$/,
				use: {
					loader: "svelte-loader",
					options: {
						emitCss: true,
						hotReload: true
					}
				}
			}
		]
	},
	mode,
	optimization: {
		chunkIds: "named",
		moduleIds: "named"
	},

	plugins: [
		new HtmlRspackPlugin({
			templateContent: () =>
				`<!doctype html>\n<html lang="en">\n<head>\n  <meta charset="utf-8" />\n  <title>App 04</title>\n</head>\n<body>\n  <div id="app_04"></div>\n</body>\n</html>`
		}),
		new ModuleFederationPlugin({
			name: "app_04",
			filename: "remoteEntry.js",
			exposes: {
				"./App": "./src/main.js",
				"./loadApp": "./src/loadApp.js"
			},
			shared: [],
			experiments: {
				asyncStartup: true
			}
		})
	],
	devtool: prod ? false : "source-map",
	experiments: {
		css: true
	}
};
