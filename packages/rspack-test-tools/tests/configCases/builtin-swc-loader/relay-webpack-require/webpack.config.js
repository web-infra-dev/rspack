const { resolve } = require("path");

module.exports = {
	resolve: {
		alias: {
			[resolve(__dirname, "./custom/MyComponent.graphql.ts")]: resolve(
				__dirname,
				"./mock.js"
			)
		},
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					rspackExperiments: {
						relay: true
					}
				}
			}
		]
	}
};
