const rspack = require("@rspack/core");
console.log("story:");
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.jsx"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset"
			}
		]
	}
};
