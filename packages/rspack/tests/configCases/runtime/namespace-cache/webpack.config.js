const path = require("path");
module.exports = {
	entry: "./index.mjs",
	resolve: {
		alias: {
			"@b": path.resolve(__dirname, "./a"),
			xx: path.resolve(__dirname, "./a"),
			ignored: path.resolve(__dirname, "./a")
		}
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				resolve: {
					alias: {
						ignored: false
					}
				}
			}
		]
	}
};
