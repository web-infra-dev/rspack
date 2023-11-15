const rspack = require("@rspack/core");
const path = require("path");

module.exports = {
	entry: {
		app: "./src/index.tsx"
	},
	devServer: {
		hot: true
	},
	resolve: {
		extensions: ["*", ".js", ".jsx", ".tsx", ".ts"],
		tsConfigPath: path.resolve(__dirname, "tsconfig.json")
	},
	output: {
		globalObject: "self",
		filename: "[name].bundle.js",
		path: path.resolve(__dirname, "dist")
	},
	module: {
		rules: [
			{
				test: /\.ttf$/,
				type: "asset/resource"
			}
		]
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html"
		})
	]
};
