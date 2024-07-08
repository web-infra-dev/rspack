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
				exclude: [/node_modules/],
				use: {
					loader: "builtin:swc-loader",
					options: {
						jsc: {
							experimental: {
								plugins: [
									[
										"@swc/plugin-prefresh", // enable prefresh specific transformation
										{}
									]
								]
							},
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
