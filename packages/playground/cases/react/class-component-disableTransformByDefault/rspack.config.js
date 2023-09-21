const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	},
	context: __dirname,
	mode: "development",
	entry: "./src/index.jsx",
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
			}
		]
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new ReactRefreshPlugin()
	],
	devServer: {
		hot: true
	},
	cache: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	watchOptions: {
		poll: 1000
	}
};
