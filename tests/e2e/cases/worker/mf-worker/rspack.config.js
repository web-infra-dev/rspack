const path = require("path")
const rspack = require("@rspack/core");

/** @type {rspack.Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	devtool: false,
	mode: "development",
	resolve: {
		alias: {
			"shared-lib": path.resolve(__dirname, "src/shared-lib")
		}
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new rspack.container.ModuleFederationPlugin({
			name: "host",
			shared: {
				"shared-lib": {
					singleton: true,
					eager: false,
					requiredVersion: false
				}
			}
		}),
	],
	devServer: {
		hot: true
	},
}
