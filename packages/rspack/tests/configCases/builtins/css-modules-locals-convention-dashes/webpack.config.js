module.exports = {
	builtins: {
		css: {
			modules: {
				localsConvention: "dashes"
			}
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
