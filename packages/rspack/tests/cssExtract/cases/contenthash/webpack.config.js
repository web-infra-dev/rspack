const { CssExtractRspackPlugin } = require("../../../../");

module.exports = [1, 2].map(n => {
	return {
		entry: "./index.js",
		module: {
			rules: [
				{
					test: /\.css$/,
					use: [CssExtractRspackPlugin.loader, "css-loader"]
				}
			]
		},
		output: {
			filename: `${n}.[name].js`
		},
		resolve: {
			alias: {
				"./style.css": `./style${n}.css`
			}
		},
		plugins: [
			new CssExtractRspackPlugin({
				filename: `${n}.[name].$[contenthash]$.css`
			})
		]
	};
});
