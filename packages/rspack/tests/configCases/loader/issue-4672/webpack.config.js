const path = require("path");

module.exports = {
	resolve: {
		tsConfigPath: path.resolve(__dirname, "./tsconfig.json")
	},
	module: {
		rules: [
			{
				test: /index/,
				loader: "./loader.js"
			}
		]
	}
};
