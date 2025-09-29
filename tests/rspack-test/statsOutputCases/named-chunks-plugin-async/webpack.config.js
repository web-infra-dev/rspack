"use strict";

const {
	ids: { NamedChunkIdsPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	optimization: { chunkIds: false },
	entry: {
		entry: "./entry"
	},
	plugins: [new NamedChunkIdsPlugin()]
};
