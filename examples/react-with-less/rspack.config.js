const rspack = require("@rspack/core");
const path = require("path");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: ["./src/index.jsx"]
	},
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
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
				test: /.less$/,
				loader: "less-loader",
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
	],
	output: {
		path: path.resolve(__dirname, "dist")
	}
};
module.exports = config;
