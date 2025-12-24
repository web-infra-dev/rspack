/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.cjs",
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
							collectTypeScriptInfo: {
								exportedEnum: true
							}
						}
					}
				]
			}
		]
	},
	optimization: {
		moduleIds: "named"
	},
	experiments: {
		inlineEnum: true
	}
};
