/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: "errors-warnings",
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.ts$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "typescript"
						}
					}
				}
			}
		]
	}
}
