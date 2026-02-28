"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true
	},
	target: "es2020",
	optimization: {
		splitChunks: {
			cacheGroups: {
				common: {
					test: /common\.js/,
					minSize: 0,
					chunks: "all",
					filename: "common.mjs"
				}
			}
		}
	}
};
