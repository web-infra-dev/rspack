/** @type {import('@rspack/core').Configuration}*/
module.exports = {
	entry: "./index.jsx",
	resolve: {
		alias: {
			"@xstyled/styled-components": "styled-components"
		},
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.jsx?$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "ecmascript",
							jsx: true
						},
						experimental: {
							plugins: [
								[
									"@swc/plugin-styled-components",
									{
										displayName: true,
										ssr: true,
										fileName: true,
										meaninglessFileNames: ["index", "styles"],
										namespace: "rspack-test",
										topLevelImportPaths: [
											"@xstyled/styled-components",
											"@xstyled/styled-components/*"
										],
										transpileTemplateLiterals: true,
										minify: true,
										pure: true,
										cssProp: true
									}
								]
							]
						}
					}
				}
			}
		]
	}
};
