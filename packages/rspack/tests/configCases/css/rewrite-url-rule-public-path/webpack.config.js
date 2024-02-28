module.exports = {
	output: {
		publicPath: "auto"
	},
	module: {
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
