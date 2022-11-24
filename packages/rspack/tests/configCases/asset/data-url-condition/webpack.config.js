module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset"
			}
		],
		parser: {
			asset: {
				dataUrlCondition: {
					maxSize: 100 * 1024
				}
			}
		}
	}
};
