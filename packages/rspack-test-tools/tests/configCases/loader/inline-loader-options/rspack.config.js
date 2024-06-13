module.exports = {
	module: {
		rules: [
			{
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "typescript",
							tsx: true
						}
					}
				}
			}
		]
	}
}
