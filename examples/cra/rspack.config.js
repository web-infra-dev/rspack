const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./src/index.jsx"
	},
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
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
				type: "asset/resource"
			}
		]
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
