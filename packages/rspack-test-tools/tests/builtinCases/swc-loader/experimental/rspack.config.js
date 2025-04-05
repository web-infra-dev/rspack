/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			type: "module"
		}
	},
	experiments: {
		outputModule: true
	},
	optimization: {
		minimize: false
	},
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
