/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolveLoader: {
		alias: {
			"my-loader": "./loader.js?query=alias"
		}
	},
	module: {
		rules: [
			{
				test: /a\.js$/,
				use: {
					loader: "./loader?query=a"
				}
			},
			{
				test: /b\.js$/,
				use: {
					loader: "my-loader"
				}
			},
			{
				test: /c\.js$/,
				use: {
					loader: "my-loader?query=c"
				}
			},
			{
				test: /d\.js$/,
				use: {
					loader: "./loader",
					options: "query=d"
				}
			},
			{
				test: /e\.js$/,
				use: {
					loader: "./loader?query=e",
					options: "query=options-e"
				}
			},
			{
				test: /f\.js$/,
				use: {
					loader: "./loader?query=f",
					options: {
						query: "options-object-f"
					}
				}
			},
			{
				test: /g\.js$/,
				use: {
					loader: "my-loader",
					options: {
						query: "options-object-g"
					}
				}
			},
			{
				test: /h\.js$/,
				use: {
					loader: "my-loader?query=h",
					options: {
						query: "options-object-h"
					}
				}
			}
		]
	}
};
