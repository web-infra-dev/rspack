const path = require("path");

function config(subpath, realContentHash = false) {
	return {
		entry: `./index.js`,
		context: path.resolve(__dirname, subpath),
		output: {
			path: path.resolve(__dirname, `dist/${subpath}`),
			filename: "[name].[contenthash].js"
		},
		optimization: {
			realContentHash,
			moduleIds: "named",
			minimize: false,
			runtimeChunk: {
				name: "runtime"
			}
		}
	};
}

/** @type {import("../../../dist").Configuration} */
module.exports = [
	config("version0"),
	config("version0-copy"),
	config("version1"),
	config("rch-version0", true),
	config("rch-version0-copy", true),
	config("rch-version1", true)
];
