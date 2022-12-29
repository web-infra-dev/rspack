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
	},
	cache: false // FIXME: build cache validate bug which needs fileDependencies on rust side
};
