module.exports = {
	stats: "errors-warnings",
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
