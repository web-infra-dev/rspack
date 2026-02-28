"use strict";

const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.ContextReplacementPlugin(
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
