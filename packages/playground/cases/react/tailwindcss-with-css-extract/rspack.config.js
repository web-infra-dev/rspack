const path = require("path");
const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

module.exports = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/main.jsx"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new ReactRefreshPlugin(),
		new rspack.CssExtractRspackPlugin()
	],
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	experiments: {
		css: false
	},
	module: {
		rules: [
			{
				test: /\.jsx$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						jsc: {
							parser: {
								syntax: "ecmascript",
								jsx: true
							},
							transform: {
								react: {
									runtime: "automatic",
									development: true,
									refresh: true
								}
							}
						}
					}
				}
			},
			{
				test: /\.css$/,
				use: [
					rspack.CssExtractRspackPlugin.loader,
					"css-loader",
					{
						loader: "postcss-loader",
						options: {
							postcssOptions: {
								plugins: {
									tailwindcss: {
										config: path.join(__dirname, "./tailwind.config.js")
									}
								}
							}
						}
					}
				]
			}
		]
	}
};
