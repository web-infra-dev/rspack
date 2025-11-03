const { rspack } = require("@rspack/core");
const { ModuleFederationPlugin } = require("@module-federation/enhanced/rspack");
const path = require("path");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	devtool: false,
	output: {
		filename: "[name].js",
		chunkFilename: "[name].chunk.js"
	},
	resolve: {
		alias: {
			"shared-lib": path.resolve(__dirname, "src/shared-lib")
		}
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new ModuleFederationPlugin({
			name: "host",
			filename: "remoteEntry.js",
			shared: {
				"shared-lib": {
					singleton: true,
					eager: false,
					requiredVersion: false
				}
			}
		})
	],
	devServer: {
		hot: false,
		port: 5680
	}
};
