module.exports = {
	// mode: "development" || "production",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "javascript/auto",
				loader: "css-loader"
			}
		]
	}
};
