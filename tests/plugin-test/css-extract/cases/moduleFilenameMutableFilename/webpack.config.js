const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		(() => {
			const self = new CssExtractRspackPlugin({ filename: "constructed.css" });

			self.options.filename = "mutated.css";

			return self;
		})()
	]
};
