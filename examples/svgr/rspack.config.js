const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./index.jsx"
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				use: ["@svgr/webpack", "url-loader"]
			}
		]
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
