/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [
					{
						loader: "less-loader",
						options: {
							additionalData: "@background: coral;"
						}
					}
				],
				type: "css",
				generator: {
					exportsOnly: false,
				}
			}
		]
	}
};
