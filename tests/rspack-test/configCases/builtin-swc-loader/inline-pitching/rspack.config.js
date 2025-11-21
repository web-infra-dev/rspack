/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript",
									jsx: false
								}
							}
						}
					}
				],
				type: "javascript/auto"
			}
		]
	}
};
