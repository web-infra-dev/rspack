"use strict";

const path = require("path");
const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new webpack.ContextReplacementPlugin(
			/replacement.c$/,
			path.resolve(__dirname, "modules"),
			{
				a: "./a",
				b: "./module-b",
				"./c": "./module-b",
				d: "d",
				"./d": "d"
			}
		)
	]
};
