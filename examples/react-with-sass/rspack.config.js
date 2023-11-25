const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: {
		main: ["./src/index.jsx"]
	},
	module: {
		rules: [
			{
				test: /\.jsx$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						sourceMap: true,
						jsc: {
							parser: {
								syntax: "ecmascript",
								jsx: true
							},
							externalHelpers: true,
							preserveAllComments: false,
							transform: {
								react: {
									runtime: "automatic",
									throwIfNamespace: true,
									useBuiltins: false
								}
							}
						}
					}
				},
				type: "javascript/auto"
			},
			{
				test: /\.s[ac]ss$/,
				loader: "sass-loader",
				type: "css"
			}
		]
	},
	optimization: {
		minimize: false // Disabling minification because it takes too long on CI
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	]
};
module.exports = config;
