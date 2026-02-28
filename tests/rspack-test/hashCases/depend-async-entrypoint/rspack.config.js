const path = require("path");

function config(subpath, filename) {
	return {
		entry: {
			main: {
				import: "./index.js",
				filename: "[name].[chunkhash].js"
			}
		},
		output: {
			path: path.resolve(__dirname, `dist/${subpath}`),
			filename
		},
		target: "node",
		optimization: {
			realContentHash: false,
			moduleIds: "named",
			chunkIds: "named",
			minimize: false
		}
	};
}

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	config("a", "[name].[chunkhash].js"),
	config("b", "[name].[contenthash].js")
];
