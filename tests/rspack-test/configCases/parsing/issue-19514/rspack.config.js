"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	output: {
		library: {
			type: "commonjs"
		}
	},
	optimization: {
		minimize: false
	}
};
