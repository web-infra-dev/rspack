"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	optimization: { chunkIds: "named" },
	entry: {
		entry: "./entry"
	},
	stats: {
		assets: true,
		modules: true,
	}
};
