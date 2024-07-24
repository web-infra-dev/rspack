const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = [1, 2].map(n => {
	return {
		entry: "./index.js",
		module: {
			rules: [
				{
					test: /\.css$/,
					use: [
						{
							loader: CssExtractRspackPlugin.loader
						},
						{
							loader: "css-loader",
							options: {
								modules: true
							}
						},
						{
							loader: "./loader",
							ident: "my-loader",
							options: {
								number: n
							}
						}
					]
				}
			]
		},
		output: {
			filename: `[name].$[contenthash]$.${n}.js`
		},
		plugins: [
			new CssExtractRspackPlugin({
				filename: `[name].$[contenthash]$.${n}.css`
			})
		]
	};
});
