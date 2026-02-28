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
	devtool: "cheap-module-source-map",
	externals: ["source-map"],
	externalsType: "commonjs",

};
