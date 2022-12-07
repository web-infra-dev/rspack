module.exports = {
	builtins: {
		css: {
			modules: {
				localIdentName: "[hash]"
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
