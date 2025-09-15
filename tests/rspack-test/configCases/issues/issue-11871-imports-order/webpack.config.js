"use strict";

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		mode: "development"
	},
	{
		mode: "production"
	},
	{
		mode: "production",
		optimization: {
			concatenateModules: false
		}
	},
	{
		mode: "development",
		optimization: {
			concatenateModules: true
		}
	}
];
