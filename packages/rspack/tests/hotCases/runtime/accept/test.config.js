module.exports = {
	entry: {
		main: "./index.js"
	},
	devServer: {
		hot: true
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
