/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		extensions: ["..."]
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								syntax: "unknown"
							}
						}
					}
				],
				type: "javascript/auto"
			}
		]
	}
};
