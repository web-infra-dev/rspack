/**
 * @type {import('webpack').Configuration | import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: "development",
	target: "web",
	devtool: "source-map",
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		},
		rules: [
			{
				test: /\.css$/,
				use: ["./loader.js", "builtin:lightningcss-loader"],
				sideEffects: true,
				type: "css/auto"
			}
		]
	},
	experiments: {
		css: true
	}
};
