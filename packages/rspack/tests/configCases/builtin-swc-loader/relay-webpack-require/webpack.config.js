const { resolve } = require("path");
const { createSwcLoaderExperiments } = require("@rspack/core");

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
					rspackExperiments: createSwcLoaderExperiments().useRelay(__dirname)
				}
			}
		]
	}
};
