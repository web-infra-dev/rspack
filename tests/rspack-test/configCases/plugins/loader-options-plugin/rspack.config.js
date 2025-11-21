var webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new webpack.LoaderOptionsPlugin({
			minimize: true
		}),
		new webpack.LoaderOptionsPlugin({
			test: /\.js$/,
			jsfile: true
		})
	]
};
