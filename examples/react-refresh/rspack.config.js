const rspack = require("@rspack/core")
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh")

const isProduction = process.env.NODE_ENV === "production"

/** @type {import('@rspack/cli').Configuration} */
const config = {
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true,
			newTreeshaking: true
		}
	},
	mode: isProduction ? "production" : "development",
	entry: { main: "./src/index.tsx" },
	devtool: 'source-map',
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
								tsx: true
							},
							transform: {
								react: {
									runtime: "automatic",
									development: !isProduction,
									refresh: !isProduction,
								}
							}
						}
					}
				}
			}
		]
	},
	optimization: {
		providedExports: true,
		minimize: false, // Disabling minification because it takes too long on CI
		sideEffects: 'flag'
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./index.html" }),
		new rspack.DefinePlugin({ "process.env.NODE_ENV": "'development'" }),
		!isProduction && new ReactRefreshPlugin(),
	].filter(Boolean)
};

module.exports = config;
