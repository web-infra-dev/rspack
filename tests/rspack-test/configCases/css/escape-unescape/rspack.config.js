"use strict";

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		target: "web",
		mode: "development",

		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	},
	{
		target: "web",
		mode: "production",

		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	}
];
