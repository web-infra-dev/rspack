var webpack = require("../../../../");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [new webpack.ContextExclusionPlugin(/dont/)]
};
