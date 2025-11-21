const { rspack } = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");
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
		new ReactRefreshPlugin(),
		new rspack.DefinePlugin({
			STUB: JSON.stringify("<div></div>")
		})
	]
};
