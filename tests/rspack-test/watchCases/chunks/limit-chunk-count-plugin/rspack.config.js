var rspack = require("@rspack/core");
/** @type {import("webpack").Configuration} */
module.exports = {
	plugins: [new rspack.optimize.LimitChunkCountPlugin({ maxChunks: 1 })]
};
