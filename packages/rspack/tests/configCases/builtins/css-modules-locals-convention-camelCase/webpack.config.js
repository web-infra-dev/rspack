module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module",
				generator: {
					exportsConvention: "camel-case",
				}
			}
		]
	}
};
