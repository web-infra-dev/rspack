const path = require("path");
const rspack = require("@rspack/core");

// Import React Refresh plugin - similar to Next.js
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

module.exports = {
	mode: "development",
	entry: "./index.js",
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "bundle.js"
	},
	module: {
		rules: [
			{
				test: /\.jsx?$/,
				exclude: /node_modules/,
				use: [
					// Add React Refresh loader first, similar to how Next.js does it with attachReactRefresh
					{
						loader: "@rspack/react-refresh-loader"
					},
					// Add SWC loader for JSX transformations
					{
						loader: "swc-loader",
						options: {
							jsc: {
								transform: {
									react: {
										runtime: "automatic",
										development: true,
										refresh: true
									}
								},
								parser: {
									syntax: "ecmascript",
									jsx: true
								}
							}
						}
					}
				]
			}
		]
	},
	experiments: {
		buildHttp: {
			allowedUris: ["https://example.com"],
			cacheLocation: path.resolve(__dirname, "lock-files"),
			httpClient: require("./mock-http-content")
		}
	},
	plugins: [
		// Add React Refresh Plugin
		new ReactRefreshPlugin(),
		// Required for HMR
		new rspack.HotModuleReplacementPlugin()
	],
	resolve: {
		extensions: [".js", ".jsx"]
	},
	devServer: {
		hot: true
	}
};
