const path = require("path");

function config(subpath) {
	return {
		entry: `./index.js`,
		context: path.resolve(__dirname, subpath),
		output: {
			path: path.resolve(__dirname, `dist/${subpath}`),
			filename: "[name].[contenthash].js"
		},
		optimization: {
			moduleIds: "named",
			minimize: false,
			runtimeChunk: {
				name: "runtime"
			}
		},
		experiments: {
			newSplitChunks: true
		}
	};
}

/** @type {import("../../../dist").Configuration} */
module.exports = [
	config("version0"),
	config("version0-copy"),
	config("version1")
];
