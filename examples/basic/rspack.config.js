const webpack = require('@rspack/core')
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		"dist/banner": "./src/index.js",
	},
	builtins: {
		html: [
			{
				template: "./index.html",
			},
		],
	},

	plugins: [
		new webpack.BannerPlugin({
			banner:
				"fullhash:[fullhash], chunkhash:[chunkhash], name:[name], base:[base], query:[query], file:[file], path:[path], ext:[ext]",
		}),
	],
};
module.exports = config;
