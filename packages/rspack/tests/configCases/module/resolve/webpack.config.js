module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css",
				resolve: {
					preferRelative: true
				}
			}
		]
	}
};
