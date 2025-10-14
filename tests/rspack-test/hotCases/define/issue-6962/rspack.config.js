"use strict";

const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new webpack.DefinePlugin({
			DEFINE_PATH: JSON.stringify("./a")
		})
	]
};
