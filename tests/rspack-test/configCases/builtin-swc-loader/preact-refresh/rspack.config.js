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
							jsx: true
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
	]
};
