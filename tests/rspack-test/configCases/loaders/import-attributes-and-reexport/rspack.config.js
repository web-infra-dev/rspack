"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				with: { type: "RANDOM" },
				use: require.resolve("./test-loader")
			}
		]
	}
};
