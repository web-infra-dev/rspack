const path = require("path");
const rspack = require("@rspack/core");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: "./src/index.ts",
	resolve: {
		tsConfigPath: path.resolve(__dirname, "tsconfig.json")
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
