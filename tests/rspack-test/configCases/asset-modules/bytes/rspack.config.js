"use strict";

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		name: "web",
		mode: "development",
		target: "web",

		module: {
			rules: [
				{
					test: /\.svg$/,
					type: "asset/bytes"
				},
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	},
	{
		name: "node",
		mode: "development",
		target: "node",
		module: {
			rules: [
				{
					test: /\.svg$/,
					type: "asset/bytes"
				},
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	},
	{
		name: "universal",
		mode: "development",
		target: ["web", "node"],
		experiments: {
			outputModule: true,
		},
		module: {
			rules: [
				{
					test: /\.svg$/,
					type: "asset/bytes"
				},
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	}
];
