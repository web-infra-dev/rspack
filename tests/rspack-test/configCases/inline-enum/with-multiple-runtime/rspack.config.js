/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a",
		b: "./b",
	},
	output: {
		filename: "[name].js",
	},
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
		chunkIds: 'named',
		concatenateModules: false,
		inlineExports: true
	},
};
