module.exports = {
	builtins: {
		css: {
			modules: {
				exportsOnly: true
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
