"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.css$/i,
				type: "css",
				extractSourceMap: true
			}
		]
	},
	experiments: {
		css: true
	}
};
