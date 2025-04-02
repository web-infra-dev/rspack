const path = require("path");
const rspack = require("@rspack/core");

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
				test: /\.js$/,
				use: {
					loader: "babel-loader",
					options: {
						presets: ["@babel/preset-env"]
					}
				}
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
	plugins: [new rspack.HotModuleReplacementPlugin()],
	devServer: {
		hot: true
	}
};
