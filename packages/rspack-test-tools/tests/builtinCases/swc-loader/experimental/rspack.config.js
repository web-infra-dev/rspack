/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: './index.js',
	output: {
		library: {
			type: "module"
		}
	},
	target: ["web", "es2020"],
	experiments: {
		outputModule: true
	},
	optimization: {
		minimize: false,
		concatenateModules: true,
	},
	devtool: false,
	module: {
		rules: [
			{
				test: /\.js$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						jsc: {
							experimental: {
								keepImportAttributes: true,
								emitAssertForImportAttributes: true
							},
							parser: {
								syntax: "typescript"
							}
						}
					}
				}
			}
		]
	},
	externals: {
		"./static-package.json": "module ./static-package.json"
	}
};
