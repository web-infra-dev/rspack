import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		(() => {
			const self = new RspackCssExtractPlugin({ filename: "constructed.css" });

			self.options.filename = "mutated.css";

			return self;
		})()
	]
};
