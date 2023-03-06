module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: "@rspack/postcss-loader",
						options: {
							pxToRem: true
						}
					}
				]
			}
		]
	}
};
