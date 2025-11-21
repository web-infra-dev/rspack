const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				type: "javascript/auto",
				test: /\.js$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						jsc: {
							parser: {
								syntax: "ecmascript",
								jsx: true,
								exportDefaultFrom: true
							}
						},
						module: {
							type: "commonjs",
							strict: false,
							strictMode: false,
							noInterop: false,
							lazy: false,
							allowTopLevelThis: true,
							ignoreDynamic: true
						}
					}
				}
			}
		]
	},
	resolve: {
		extensions: [".native.js", ".js"]
	},
	plugins: [
		new rspack.container.ModuleFederationPluginV1({
			name: "test",
			shared: {
				pkg: {
					singleton: true,
					eager: true
				}
			}
		})
	]
};
