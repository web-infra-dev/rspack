const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: {
		main: ["./src/index.jsx"]
	},
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/,
				use: ["sass-loader"],
				type: "css"
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
