const path = require("path");

module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				resourceQuery: /raww/,
				type: "asset/source"
			}
		]
	},
	resolve: {
		alias: {
			"./answer": path.resolve(__dirname, "./answer.js?raww"),
			"./no-query-answer": path.resolve(__dirname, "./answer.js")
		}
	}
};
