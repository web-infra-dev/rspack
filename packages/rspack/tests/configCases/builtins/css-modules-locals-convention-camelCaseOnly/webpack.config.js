module.exports = {
	builtins: {
		css: {
			modules: {
				localsConvention: "camelCaseOnly"
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
