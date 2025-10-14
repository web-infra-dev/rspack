"use strict";

const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new webpack.DefinePlugin({
			PROPERTY: JSON.stringify("foo")
		})
	]
};
