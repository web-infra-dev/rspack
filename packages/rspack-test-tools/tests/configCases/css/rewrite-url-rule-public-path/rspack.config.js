/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: 'web',
node: {
  __dirname: false,
  __filename: false
},
	output: {
		publicPath: "auto"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		},
		rules: [
			{
				test: /\.png$/i,
				type: "asset/resource",
				generator: {
					filename: "[name][ext]",
					publicPath: "https://test.rspack.dev/cdn/"
				}
			}
		]
	},
	experiments: {
		css: true
	}
};
