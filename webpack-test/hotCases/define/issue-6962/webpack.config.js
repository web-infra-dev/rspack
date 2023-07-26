"use strict";

const webpack = require("../../../../");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new webpack.DefinePlugin({
			DEFINE_PATH: JSON.stringify("./a")
		})
	]
};
