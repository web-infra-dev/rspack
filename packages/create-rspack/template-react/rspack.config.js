const { HtmlRspackPlugin } = require("@rspack/cli");

/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.jsx"
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset"
			}
		]
	},
	plugins: [new HtmlRspackPlugin({ template: "./index.html" })]
};
