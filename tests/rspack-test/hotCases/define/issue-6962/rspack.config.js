"use strict";

const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.DefinePlugin({
			DEFINE_PATH: JSON.stringify("./a")
		})
	]
};
