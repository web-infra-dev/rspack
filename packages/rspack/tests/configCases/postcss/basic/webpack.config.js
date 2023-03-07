module.exports = {
	module: {
		defaultRules: [],
		rules: [
			{
				test: /\.module\.css$/,
				use: [
					{
						loader: "@rspack/postcss-loader",
						options: {
							modules: true
						}
					}
				],
				type: "css"
			},
			{
				test: /\.css$/,
				use: [
					{
						loader: "@rspack/postcss-loader"
					}
				],
				type: "css"
			}
		]
	}
};
