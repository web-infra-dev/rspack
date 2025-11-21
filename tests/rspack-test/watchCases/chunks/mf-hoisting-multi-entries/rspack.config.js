const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.js",
	},
	output: {
		filename: "[name].js",
	},
	target: "web",
	optimization: {
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				defaultVendors: {
					name: "vendors", // add name for defaultVendors, need a specific name to run it at test.config.js findBundle
					reuseExistingChunk: true,
					test: /[\\/]node_modules[\\/]/i,
					priority: -10
				}
			},
		}
	},
	module: {
		rules: [
			{
				test: /\.js/,
				use: [{
					loader: 'builtin:swc-loader',
					options: {
						jsc: {
							parser: {
								jsx: true,
								syntax: 'ecmascript'
							},
							transform: {
								react: {
									development: true,
									refresh: true,
									runtime: 'automatic'
								}
							},
							target: "es2022"
						},
					}
				}],
			}
		]
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "test",
			shareStrategy: "loaded-first"
		}),
		new ReactRefreshPlugin() // Need this to trigger hoisting (hoist_container_references_plugin.rs)
	]
};
