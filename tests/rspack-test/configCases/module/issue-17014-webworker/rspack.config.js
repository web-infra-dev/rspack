"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true
	},
	experiments: {
		},
	target: ["web", "es2020"],
	optimization: {
		splitChunks: {
			minSize: 1,
			maxSize: 1
		}
	}
};
