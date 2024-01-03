const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./src/index.jsx"
	},
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.(png|svg|jpg)$/,
				type: "asset/resource"
			},
			{
				test: /\.jsx$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "ecmascript",
							jsx: true
						},
						transform: {
							react: {
								runtime: "classic"
							}
						}
					}
				}
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
