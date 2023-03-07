module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.scss$/,
				use: [
					{
						loader: "@rspack/postcss-loader",
						options: {
							pxToRem: true
						}
					},
					{ loader: "sass-loader" }
				],
				type: "css"
			}
		]
	}
};
