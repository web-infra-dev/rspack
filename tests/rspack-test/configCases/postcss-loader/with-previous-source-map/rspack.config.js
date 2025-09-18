const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	devtool: "source-map",
	node: false,
	module: {
		rules: [
			{
				test: /\.scss$/,
				use: [
					{
						loader: "postcss-loader",
						options: {
							postcssOptions: {
								plugins: [require.resolve("postcss-pxtorem")]
							}
						}
					},
					{
						loader: "sass-loader",
						options: {
							// use legacy API to generate source maps
							api: "legacy",
							sassOptions: {
								silenceDeprecations: ["legacy-js-api"]
							}
						}
					}
				],
				type: "css",
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
	],
	experiments: {
		css: true
	}
};
