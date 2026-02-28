const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [
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
	devtool: "source-map",
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	],

};
