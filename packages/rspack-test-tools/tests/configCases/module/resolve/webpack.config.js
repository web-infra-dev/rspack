module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		},
		rules: [
			{
				test: /\.png$/,
				type: "asset"
			}
		]
	}
};
