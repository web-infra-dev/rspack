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
								target: "es2015",
								parser: {
									syntax: "ecmascript",
									jsx: true,
									dynamicImport: true,
									classProperty: true,
									exportNamespaceFrom: true,
									exportDefaultFrom: true
								}
							}
						}
					}
				]
			}
		]
	},
	devtool: "inline-source-map"
};
