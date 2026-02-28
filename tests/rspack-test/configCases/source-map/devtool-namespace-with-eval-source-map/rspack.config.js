const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		"entry-a": [path.join(__dirname, "./src/entry-a")],
		"entry-b": [path.join(__dirname, "./src/entry-b")]
	},

	output: {
		filename: "[name]-bundle.js",
		library: { type: "commonjs", name: "library-[name]" },
		devtoolNamespace: "library-[name]"
	},
	devtool: "eval-source-map"
};
