"use strict";

const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /a\.js$/,
				use: ["./queryloader?lions=roar"]
			}
		]
	},
	plugins: [
		new rspack.ContextReplacementPlugin(
			/replacement.d$/,
			path.resolve(__dirname, "modules?cats=meow"),
			{
				a: path.resolve(__dirname, "./modules/a")
			}
		)
	]
};
