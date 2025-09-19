const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "source-map",
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
	},
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
