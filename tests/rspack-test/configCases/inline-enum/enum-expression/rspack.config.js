/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	resolve: {
		extensions: [".ts", "..."]
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
									syntax: "typescript"
								}
							},
							rspackExperiments: {
								collectTypeScriptInfo: {
									exportedEnum: true
								}
							}
						}
					}
				]
			}
		]
	},
	experiments: {
		inlineEnum: true
	}
};
