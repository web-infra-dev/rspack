module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module",
				generator: {
					localIdentName: "[hash]"
				}
			}
		]
	}
};
