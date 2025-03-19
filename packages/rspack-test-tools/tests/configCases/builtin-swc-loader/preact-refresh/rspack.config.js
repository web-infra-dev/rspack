const { rspack } = require("@rspack/core");
const PreactRefreshPlugin = require("@rspack/plugin-preact-refresh");
const { ConcatSource, RawSource } = require("webpack-sources");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.jsx",
	mode: "development",
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.jsx$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "ecmascript",
							jsx: true,
							sourceMap: true
						},
						transform: {
							react: {
								runtime: "classic",
								pragma: "React.createElement",
								pragmaFrag: "React.Fragment",
								throwIfNamespace: true,
								useBuiltins: false
							}
						}
					}
				}
			}
		]
	},
	plugins: [
		new rspack.HotModuleReplacementPlugin(),
		new PreactRefreshPlugin(),
		new rspack.DefinePlugin({
			STUB: JSON.stringify("<div></div>")
		}),
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("_", compilation => {
					compilation.hooks.processAssets.tap("_", assets => {
						compilation.updateAsset(
							"bundle0.js",
							new ConcatSource(
								new RawSource("const self = globalThis;"), // mock self to NodeJs specific global object
								assets["bundle0.js"]
							)
						);
					});
				});
			}
		}
	]
};
