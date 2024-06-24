const ReactRefreshRspackPlugin = require("../../../..");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	context: __dirname,
	entry: "./index.jsx",
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"],
		alias: {
			react: "preact/compat",
			"react-dom/test-utils": "preact/test-utils",
			"react-dom": "preact/compat", // Must be below test-utils
			"react/jsx-runtime": "preact/jsx-runtime"
		}
	},
	module: {
		rules: [
			{
				test: /\.jsx?$/,
				exclude: [/node_modules/, /rspack-plugin-preact-refresh[\\\/]client/],
				use: {
					loader: "builtin:swc-loader",
					options: {
						rspackExperiments: {
							preact: {} // enable preact swc plugin
						},
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
									pragma: "h",
									pragmaFrag: "Fragment",
									throwIfNamespace: true,
									useBuiltins: false,
									refresh: true // enable react hooks hmr compatiblity
								}
							}
						}
					}
				},
				type: "javascript/auto"
			}
		]
	},
	plugins: [new ReactRefreshRspackPlugin()]
};
