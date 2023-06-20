module.exports = {
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
							},
							experimental:{
								plugins:[
									[
										"@swc/plugin-styled-components",
										{
											"displayName": true,
											"ssr": true,
											"fileName":true,
											"namespace": "my-app"
										}
									]
								]
							},
						},
					}
				},
				type: "javascript/auto"
			}
		]
	},
	stats: "errors-warnings"
};
