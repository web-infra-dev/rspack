"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		target: "web",
		optimization: {
			chunkIds: "named"
		},
		experiments: {
			css: true
		}
	},
	// {
	// 	target: "web",
	// 	optimization: {
	// 		chunkIds: "named"
	// 	},
	// 	experiments: {
	// 		css: true,
	// 		outputModule: true
	// 	}
	// }
];
