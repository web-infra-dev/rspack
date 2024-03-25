/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./src/index",
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
