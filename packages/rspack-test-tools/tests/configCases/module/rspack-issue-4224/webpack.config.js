/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				oneOf: [
					{
						test: /\.s[ac]ss$/i,
						oneOf: [
							{
								test: /\.module\.\w+$/i,
								exclude: [/index\.scss$/],
								use: [{ loader: "sass-loader" }],
								type: "css/module"
							},
							{
								exclude: [/index\.scss$/],
								use: [{ loader: "sass-loader" }],
								type: "css"
							}
						]
					},
					{
						exclude: [
							/\.(js|mjs|cjs|jsx)$/,
							/\.(ts|mts|cts|tsx)$/,
							/\.html$/,
							/\.json$/
						],
						type: "asset/resource"
					}
				]
			}
		]
	}
};
