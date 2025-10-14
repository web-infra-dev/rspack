"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	node: {
		__filename: "eval-only",
		__dirname: "eval-only"
	}
};
