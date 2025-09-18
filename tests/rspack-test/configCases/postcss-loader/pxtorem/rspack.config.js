/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: "postcss-loader",
						options: {
							postcssOptions: {
								plugins: [require.resolve("postcss-pxtorem")]
							}
						}
					}
				],
				type: "css/auto",
				generator: {
					exportsOnly: false
				}
			}
		]
	}
};
