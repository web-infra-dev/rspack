module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				uses: [
					{
						builtinLoader: "sass-loader",
						options: {
							additionalData: "$prepended-data: hotpink;"
						}
					}
				],
				type: "css"
			}
		]
	}
};
