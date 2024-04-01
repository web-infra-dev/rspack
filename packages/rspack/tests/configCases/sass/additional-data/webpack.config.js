module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [
					{
						loader: "sass-loader",
						options: {
							additionalData: "$prepended-data: hotpink;"
						}
					}
				],
				type: "css",
				generator: {
					exportsOnly: false,
				}
			}
		]
	}
};
