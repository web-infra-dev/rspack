module.exports = {
	builtins: {
		css: {
			modules: true
		}
	},
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module"
			}
		]
	}
};
