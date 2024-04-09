const rspack = require("@rspack/core");
module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: "postcss-loader",
						options: {
							postcssOptions: {
								plugins: ["postcss-pxtorem"]
							}
						}
					}
				],
				type: "css/auto",
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
