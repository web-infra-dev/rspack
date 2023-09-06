const { HtmlPlugin } = require("@rspack/core");

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
	plugins: [new HtmlPlugin({ template: "./index.html" })]
};
