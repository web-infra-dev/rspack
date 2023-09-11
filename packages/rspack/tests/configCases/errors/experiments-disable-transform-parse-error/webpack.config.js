module.exports = {
	module: {
		rules: [
			{
				test: /\.jsx?$/,
				type: "javascript/auto"
			}
		]
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	}
};
