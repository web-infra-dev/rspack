module.exports = {
	builtins: {
		css: {
			modules: true
		}
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module"
			}
		]
	}
};
