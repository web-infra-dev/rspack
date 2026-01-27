const path = require("path");

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		entry: {
			index: path.resolve(__dirname, "./index.js")
		},
		output: {
			library: {
				name: "[name]",
				export: "default",
			},
		},
		optimization: {
			concatenateModules: true
		}
	},
	{
		entry: {
			index: path.resolve(__dirname, "./index.js")
		},
		output: {
			library: {
				name: "[name]_doc",
				export: "default"
			},
		},
		optimization: {
			concatenateModules: true
		}
	}
];
