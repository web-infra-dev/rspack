/** @type {import("../../../../").Configuration} */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.[tj]sx?$/,
				loader: "builtin:swc-loader",
				options: {
					parser: {
						syntax: "typescript",
						tsx: true
					}
				}
			},
			{
				test: /\.[tj]sx?$/,
				enforce: "pre",
				loader: "source-map-loader"
			}
		]
	}
};
