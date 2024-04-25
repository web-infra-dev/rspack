const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.js"
	},
	output: {
		filename: "[name]/index.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				aStyles: {
					type: "css/mini-extract",
					name: "a",
					chunks: chunk => chunk.name === "a",
					enforce: true
				},
				bStyles: {
					type: "css/mini-extract",
					name: "b",
					chunks: chunk => chunk.name === "b",
					enforce: true
				}
			}
		}
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name]/index.css"
		})
	]
};
