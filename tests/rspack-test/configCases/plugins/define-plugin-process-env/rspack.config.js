"use strict";

const DefinePlugin = require("@rspack/core").DefinePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	mode: "production",
	plugins: [
		new DefinePlugin({
			"process.env.ENVIRONMENT": JSON.stringify("node")
		})
	]
};
