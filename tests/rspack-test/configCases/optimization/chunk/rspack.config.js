const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "deterministic"
	},
};
