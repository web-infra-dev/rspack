/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	module: {
		parser: {
			"css/auto": {
				namedExports: true
			}
		},
		rules: [
			{
				test: /\.css$/,
				generator: {
					exportsOnly: false
				},
				use: [
					{
						loader: "builtin:lightningcss-loader",
						/** @type {import("@rspack/core").LightningcssLoaderOptions} */
						options: {
							unusedSymbols: ["unused"],
							targets: "ie 10",
							exclude: {
								nesting: true
							}
						}
					}
				],
				type: "css/auto"
			}
		]
	},
	experiments: {
		css: true
	}
};
