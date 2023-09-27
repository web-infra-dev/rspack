module.exports = {
	entry: "./index.js",
	mode: "development",
	module: {
		rules: [
			{
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript"
								}
							}
						},
						ident: "builtin-swc-loader"
					}
				]
			}
		]
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	}
};
