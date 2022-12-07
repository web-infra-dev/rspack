module.exports = {
	builtins: {
		css: {
			modules: {
				localIdentName: "[path][name]__[local]"
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
