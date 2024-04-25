module.exports = {
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
	}
};
