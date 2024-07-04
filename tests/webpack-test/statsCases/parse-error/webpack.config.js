"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: {
		timings: false,
		builtAt: false,
		hash: false,
		modules: true,
		chunks: false
	}
};
