module.exports = {
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							// Enable source map
							sourceMap: true,
							jsc: {
								parser: {
									syntax: "typescript",
									jsx: false
								}
							}
						}
					}
				],
				type: "javascript/auto"
			}
		]
	}
};
