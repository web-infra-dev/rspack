"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "eval-source-map",
	target: "node",
	experiments: {
		outputModule: true
	}
};
