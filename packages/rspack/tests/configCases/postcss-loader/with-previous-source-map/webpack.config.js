const rspack = require("@rspack/core");
module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.scss$/,
				use: [
					{
						loader: "postcss-loader",
						options: {
							postcssOptions: {
								plugins: ["postcss-pxtorem"]
							}
						}
					},
					{
						loader: "sass-loader"
					}
				],
				type: "css",
				generator: {
					exportsOnly: false,
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
