module.exports = {
	entry: "./index",
	context: "./src",
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [
					{
						loader: "less-loader"
					}
				],
				type: "css"
			}
		]
	}
};
