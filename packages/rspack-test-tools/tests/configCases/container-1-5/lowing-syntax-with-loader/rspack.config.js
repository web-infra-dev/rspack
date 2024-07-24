const { rspack } = require("@rspack/core");
const { ModuleFederationPlugin } = rspack.container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								target: "es6"
							}
						}
					}
				]
			}
		]
	},
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				format: {
					ecma: 6
				}
			})
		]
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container.js",
			library: { type: "commonjs-module" },
			exposes: ["./module"]
		})
	]
};
