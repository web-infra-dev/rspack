const path = require("path");

function config(subpath) {
	return {
		mode: "production",
		entry: "./index.js",
		context: path.resolve(__dirname, subpath),
		output: {
			path: path.resolve(__dirname, `dist/${subpath}`),
			filename: "main.[fullhash].js",
			cssFilename: "main.[contenthash].css"
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	};
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [config("version0"), config("version1")];
