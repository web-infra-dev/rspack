/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		},
		rules: [
			{
				test: /\.css$/,
				type: 'css/auto'
			}
		]
	},
	devtool: "source-map",
	externals: ["source-map"],
	externalsType: "commonjs"
};
