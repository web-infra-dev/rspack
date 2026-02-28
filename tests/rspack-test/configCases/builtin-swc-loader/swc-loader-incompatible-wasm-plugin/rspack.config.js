const { rspack } = require("@rspack/core");
const path = require("path");
const { RawSource } = require("webpack-sources");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.jsx"
	},
	resolve: {
		extensions: ["...", ".jsx"],
		alias: {
			"@swc/helpers": path.dirname(require.resolve("@swc/helpers/package.json"))
		}
	},
	module: {
		rules: [
			{
				test: /\.(jsx|js)$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						// Enable source map
						sourceMaps: true,
						jsc: {
							target: "es5",
							parser: {
								syntax: "ecmascript",
								jsx: true
							},
							externalHelpers: true,
							preserveAllComments: false,
							transform: {
								react: {
									runtime: "automatic",
									pragma: "React.createElement",
									pragmaFrag: "React.Fragment",
									throwIfNamespace: true,
									useBuiltins: false
								}
							},
							experimental: {
								cacheRoot: __dirname + "/.swc",
								plugins: [
									[
										__dirname + "/node_modules/swc-wasm-plugin",
										{
											exclude: ["error"]
										}
									]
								]
							}
						}
					}
				},
				type: "javascript/auto"
			},
			{
				test: /\.(png|svg|jpg)$/,
				type: "asset/resource"
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	optimization: {
		minimize: false // Disabling minification because it takes too long on CI
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		}),
		{
			// Replace all assets with empty content to avoid evaluation that causes errors
			apply(compiler) {
				compiler.hooks.compilation.tap("_", compilation => {
					compilation.hooks.processAssets.tap("_", assets => {
						let names = Object.keys(assets);
						names.forEach(name => {
							assets[name] = new RawSource("");
						});
					});
				});
			}
		}
	],
};
module.exports = config;
