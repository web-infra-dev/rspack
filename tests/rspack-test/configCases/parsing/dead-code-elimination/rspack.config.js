"use strict";

const fs = require("fs");
const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		optimization: {
			minimize: false
		},
		module: {
			rules: [
				{
					test: /index.js$/,
					type: "javascript/dynamic"
				},
				{
					test: /esm/,
					type: "javascript/esm"
				}
			]
		},
		plugins: [
			{
				apply(compiler) {
					compiler.hooks.compilation.tap("Test", (compilation) => {
						compilation.hooks.processAssets.tap(
							{
								name: "copy-webpack-plugin",
								stage:
									compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
							},
							() => {
								const data = fs.readFileSync(
									path.resolve(__dirname, "./test.js")
								);

								compilation.emitAsset(
									"test.js",
									new rspack.sources.RawSource(data)
								);
							}
						);
					});
				}
			}
		]
	}
];
