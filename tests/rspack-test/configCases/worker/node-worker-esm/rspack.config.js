"use strict";

/** @type {import("../../../../").Configuration} */
module.exports = {
	target: "node",
	entry: "./index.js",
	optimization: {
		chunkIds: "named",
	},
	output: {
		module: true,
		filename: "bundle.mjs"
	},
	experiments: {
		}
};
