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
	builtins: {
		css: {
			exportsOnly: false
		}
	},
	devServer: {
		hot: true
	}
};
