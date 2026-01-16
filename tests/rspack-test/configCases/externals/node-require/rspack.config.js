"use strict";

const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "commonjs2" }
	},
	externals: {
		external: ["webpack", "version"]
	},
	plugins: [
		new webpack.DefinePlugin({
			NODE_VERSION: JSON.stringify(process.version),
			EXPECTED: JSON.stringify(webpack.version)
		})
	]
};
