const NodePolyfillPlugin = require("@rspack/plugin-node-polyfill");
const path = require("path");

function config(subpath, realContentHash) {
	return {
		mode: "development",
		devtool: false,
		context: __dirname,
		entry: "./index.js",
		output: {
			path: path.resolve(__dirname, `./dist/${subpath}`),
			filename: "[name].[contenthash]-[contenthash:6].js"
		},
		optimization: {
			realContentHash
		},
		plugins: [new NodePolyfillPlugin()]
	};
}

/** @type {import("../../../").Configuration} */
module.exports = [
	config("a1", false),
	config("b1", false),
	config("c1", false),
	config("d1", false),
	config("a1", true),
	config("b1", true),
	config("c1", true),
	config("d1", true)
];
