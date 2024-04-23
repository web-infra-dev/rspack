module.exports = {
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [{ loader: "less-loader" }],
				type: "css"
			}
		]
	}
};
