/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./src/index",
	target: 'web',
	node: false,
	output: {
		publicPath: '/'
	},
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [
					{
						loader: "less-loader"
					}
				],
				type: "css",
				generator: {
					exportsOnly: false
				}
			}
		]
	}
};
