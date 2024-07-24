const path = require("path")
module.exports = {
	module: {
		rules: [
			{
				test: path.resolve(__dirname, "index.js"),
				loader: "./loader"
			}
		]
	},
}
