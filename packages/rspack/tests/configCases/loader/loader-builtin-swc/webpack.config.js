module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.js$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						// Enable source map
						sourceMap: true,
						jsc: {
							parser: {
								syntax: "typescript",
								jsx: true
							}
						}
					}
				},
				type: "javascript/auto"
			}
		]
	}
};
