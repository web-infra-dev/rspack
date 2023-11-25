const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./src/index.tsx"
	},
	module: {
		rules: [
			{
				test: /\.tsx$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						sourceMap: true,
						jsc: {
							parser: {
								syntax: "typescript",
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
				test: /\.ts$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						sourceMap: true,
						jsc: {
							parser: {
								syntax: "typescript"
							},
							externalHelpers: true,
							preserveAllComments: false
						}
					}
				},
				type: "javascript/auto"
			},
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	},
	optimization: {
		minimize: false // Disabling minification because it takes too long on CI
	},
	plugins: [
		new ReactRefreshPlugin(),
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		}),
		new rspack.CopyRspackPlugin({
			patterns: [
				{
					from: "public"
				}
			]
		})
	]
};
module.exports = config;
