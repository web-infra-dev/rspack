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
		inlineExports: true
	}
};
