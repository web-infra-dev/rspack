"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		target: "web",
		optimization: {
			chunkIds: "named"
		},

		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	},
];
