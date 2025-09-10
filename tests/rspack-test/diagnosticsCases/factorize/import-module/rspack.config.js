const path = require("path");

module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: path.resolve(__dirname, "./example-loader")
			}
		]
	},
	resolveLoader: {
		alias: {
			"import-module-example": path.resolve(
				__dirname,
				"./import-module-example-loader"
			)
		}
	}
};
