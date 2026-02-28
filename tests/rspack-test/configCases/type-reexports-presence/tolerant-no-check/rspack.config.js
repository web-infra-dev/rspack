module.exports = /** @type {import("@rspack/core").Configuration} */ ({
	entry: "./index.ts",
	resolve: {
		extensions: ["...", ".ts"]
	},
	module: {
		parser: {
			javascript: {
				typeReexportsPresence: "tolerant-no-check"
			}
		},
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
							collectTypeScriptInfo: {}
						}
					}
				]
			}
		]
	},
});
