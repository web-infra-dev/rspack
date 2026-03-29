const path = require("path");
const rspack = require("@rspack/core");

function builtinConfig(subpath) {
	return {
		mode: "production",
		entry: "./index.js",
		context: path.resolve(__dirname, subpath),
		output: {
			path: path.resolve(__dirname, `dist/builtin-${subpath}`),
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

function extractConfig(subpath) {
	return {
		mode: "production",
		entry: "./index.js",
		context: path.resolve(__dirname, subpath),
		output: {
			path: path.resolve(__dirname, `dist/extract-${subpath}`),
			filename: "main.[fullhash].js"
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					use: [rspack.CssExtractRspackPlugin.loader, "css-loader"],
					type: "javascript/auto"
				}
			]
		},
		plugins: [
			new rspack.CssExtractRspackPlugin({
				filename: "main.[contenthash].css"
			})
		],
		optimization: {
			realContentHash: true
		}
	};
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	builtinConfig("version0"),
	builtinConfig("version1"),
	extractConfig("version0"),
	extractConfig("version1")
];
