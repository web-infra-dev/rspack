module.exports = {
	devtool: "source-map",
	externals: ["source-map"],
	externalsType: "commonjs",
	module: {
		rules: [
			{
				test: /\.jsx$/,
				loader: "builtin:swc-loader",
				options: {
					sourceMap: true,
					jsc: {
						parser: {
							syntax: "ecmascript",
							jsx: true
						}
					}
				}
			}
		]
	}
};
