module.exports = {
	builtins: {
		css: {
			modules: true
		}
	},
	module: {
		rules: [
			{
				test: /\.scss$/,
				use: [{ builtinLoader: "sass-loader" }],
				type: "css/module"
			}
		]
	}
};
