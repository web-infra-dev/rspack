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
								},
								target: "esnext"
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
