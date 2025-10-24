/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
									exportedEnum: "const-only"
								}
							}
						}
					}
				]
			}
		]
	},
	optimization: {
		moduleIds: "named",
		concatenateModules: false
	},
	experiments: {
		inlineEnum: true
	}
};
