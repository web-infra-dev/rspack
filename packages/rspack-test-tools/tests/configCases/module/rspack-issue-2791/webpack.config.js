const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				resourceQuery: /raw/,
				type: "asset/source"
			}
		]
	},
	resolve: {
		alias: {
			"./answer": path.resolve(__dirname, "./answer.js?raw"),
			"./no-query-answer": path.resolve(__dirname, "./answer.js")
		}
	}
};
