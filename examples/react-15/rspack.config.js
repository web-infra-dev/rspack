module.exports = {
	entry: {
		main: "./src/index.jsx"
	},
	module: {
		rules: [
			{
				test: /\.(png|svg|jpg)$/,
				type: "asset/resource"
			}
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
