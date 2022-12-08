module.exports = {
	builtins: {
		css: {
			modules: {
				localsConvention: "camelCase"
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
