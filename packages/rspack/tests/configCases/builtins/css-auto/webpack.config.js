module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto",
				generator: {
					exportsOnly: false,
				}
			}
		]
	}
};
