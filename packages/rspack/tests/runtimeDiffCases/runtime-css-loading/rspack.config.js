module.exports = {
	output: {
		publicPath: "/public/",
		cssFilename: "css/[name].css"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module"
			}
		]
	},
	devServer: {
		hot: true
	}
};
