"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	target: "web",
	output: {
		module: true,
		library: {
			type: "module"
		}
	},
	module: {
		parser: {
			javascript: {
				exportsPresence: "error",
			}
		},
		rules: [
			{
				test: /\.custom$/i,
				loader: require.resolve("./loader")
			}
		]
	},
};
