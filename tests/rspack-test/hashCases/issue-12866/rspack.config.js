const path = require("path");

function config(subpath) {
	return {
		mode: "production",
		context: path.resolve(__dirname, subpath),
		entry: "./index.js",
		experiments: { css: true },
		module: { rules: [{ test: /\.css$/, type: "css/auto" }] },
		output: {
			path: path.resolve(__dirname, `dist/`),
			filename: "main.[fullhash].js",
			cssFilename: "main.[contenthash].css"
		},
		optimization: { realContentHash: true, minimize: false }
	};
}

module.exports = [config("version0"), config("version1")];
