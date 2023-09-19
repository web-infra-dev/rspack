module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				test: /index\.js/,
				use: [
					{
						loader: "./test-loader.js"
					}
				]
			}
		]
	}
};
