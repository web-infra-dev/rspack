const CopyPlugin = require("copy-webpack-plugin");
/*
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		main: "./src/index.tsx"
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		copy: {
			patterns: [
				{
					from: "public"
				}
			]
		}
	}
};
