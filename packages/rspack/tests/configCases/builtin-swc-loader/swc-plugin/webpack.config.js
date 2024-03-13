module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.js$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						sourceMap: true,
						jsc: {
							parser: {
								syntax: "typescript",
								jsx: true
							},
							experimental: {
								plugins: [
									[
										"@swc/plugin-remove-console",
										{
											"exclude": ["error"]
										}
									]
								],
							},
						},
					},
				},
				type: "javascript/auto"
			}
		]
	}
};
