const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["web", "browserslist:chrome > 95"],
	node: {
		__dirname: false,
		__filename: false,
	},
	optimization: {
		minimize: true,
	},
};
