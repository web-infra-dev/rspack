const webpack = require("../../../../");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: false
	},
	plugins: [new webpack.ids.DeterministicChunkIdsPlugin()]
};
