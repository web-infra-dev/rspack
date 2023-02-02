module.exports = {
	output: {
		cssFilename: "css/[name].css"
	},
	module: {
		rules: [
			{
				test: /\.png$/i,
				type: "asset",
				generator: {
					filename: "image/[name].[contenthash:8][ext]"
				}
			}
		]
	}
};
