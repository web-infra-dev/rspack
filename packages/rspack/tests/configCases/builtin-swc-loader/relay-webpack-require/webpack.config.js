const { resolve } = require("path");

module.exports = {
	resolve: {
		alias: {
			[resolve(__dirname, "./custom/MyComponent.graphql.ts")]: resolve(
				__dirname,
				"./mock.js"
			)
		}
	},
	module: {
		rules: [
			{
				test: /\.jsx?$/,
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
