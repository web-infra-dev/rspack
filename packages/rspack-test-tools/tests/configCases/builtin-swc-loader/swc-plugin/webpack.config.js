/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.js$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
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
