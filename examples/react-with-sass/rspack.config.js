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
	optimization: {
		minimize: false, // Disabling minification because it takes too long on CI
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
