const rspack = require("@rspack/core");
console.log("story:");
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.jsx"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	optimization: {
		minimize: false // Disabling minification because it takes too long on CI
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
				test: /\.svg$/,
				type: "asset"
			}
		]
	}
};
