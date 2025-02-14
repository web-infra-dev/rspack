const path = require("path");
const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

const prod = process.env.NODE_ENV === "production";
/** @type {import("@rspack/cli").Configuration} */
module.exports = {
	resolve: {
		extensions: [".js", ".jsx"]
	},
	entry: { main: "./index.jsx" },
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: path.resolve(__dirname, "./index.html")
		}),
		!prod && new ReactRefreshPlugin()
	].filter(Boolean),
	module: {
		rules: [
			{
				test: /\.(j|t)s$/,
				exclude: [/[\\/]node_modules[\\/]/],
				loader: "builtin:swc-loader",
				options: {
					sourceMap: true,
					jsc: {
						parser: {
							syntax: "typescript"
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			},
			{
				test: /\.(j|t)sx$/,
				loader: "builtin:swc-loader",
				exclude: [/[\\/]node_modules[\\/]/],
				options: {
					sourceMap: true,
					jsc: {
						parser: {
							syntax: "typescript",
							tsx: true
						},
						transform: {
							react: {
								runtime: "automatic",
								development: !prod,
								refresh: !prod
							}
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			}
		]
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				d1: {
					test: /\/d1\//
				}
			}
		}
	}
};
