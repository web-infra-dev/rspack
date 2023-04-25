const EslintPlugin = require("eslint-rspack-plugin");
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: "development",
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	module: {
		rules: [
			{
				test: /src/,
				exclude: /node_modules/,
				enforce: "pre",
				use: [
					{
						loader: "eslint-loader"
					}
				]
			}
		]
	}
};
