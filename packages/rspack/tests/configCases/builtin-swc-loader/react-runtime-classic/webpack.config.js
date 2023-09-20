module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						transform: {
							react: {
								runtime: "classic"
							}
						}
					}
				}
			}
		]
	}
};
