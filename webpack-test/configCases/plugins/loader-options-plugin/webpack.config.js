var webpack = require("../../../../");

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
