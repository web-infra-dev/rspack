module.exports = {
	builtins: {
		treeShaking: true,
		sideEffects: true
	},
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				use: [
					{
						loader: "@rspack/postcss-loader",
						options: {
							modules: true
						}
					}
				]
			},
			{
				test: /\.css$/,
				use: [
					{
						loader: "@rspack/postcss-loader"
					}
				]
			}
		]
	}
};
