/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		hashDigestLength: 8
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "javascript/auto",
				use: [
					"style-loader",
					{
						loader: "css-loader",
						options: {
							modules: {
								namedExport: false,
								localIdentName: "[name]__[local]--[contenthash]",
								exportLocalsConvention: "camel-case"
							}
						}
					}
				]
			}
		]
	},
	experiments: {
		css: false
	}
};
