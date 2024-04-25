const path = require("path");

const base = {
	mode: "production",
	entry: {
		index: {
			import: "./index",
			runtime: "runtime"
		},
		a: "./a",
		b: "./b"
	},
	module: {
		rules: [
			{
				test: /\.(png|jpg)$/,
				type: "asset/resource"
			}
		]
	},
	optimization: {
		realContentHash: true
	},
	stats: "normal"
};

/** @type {import("../../../").Configuration[]} */
module.exports = [
	{
		...base,
		name: "a-normal",
		context: path.resolve(__dirname, "a"),
		devtool: false,
		output: {
			path: path.resolve(__dirname, "./dist/a-normal"),
			filename: "[name].[contenthash]-[contenthash:6].js",
			assetModuleFilename: "[contenthash][ext]"
		}
	},
	{
		...base,
		name: "b-normal",
		context: path.resolve(__dirname, "b"),
		devtool: false,
		output: {
			path: path.resolve(__dirname, "./dist/b-normal"),
			filename: "[name].[contenthash]-[contenthash:6].js",
			assetModuleFilename: "[contenthash][ext]"
		}
	},
	{
		...base,
		name: "a-source-map",
		context: path.resolve(__dirname, "a"),
		devtool: "source-map",
		output: {
			path: path.resolve(__dirname, "./dist/a-source-map"),
			filename: "[name].[contenthash]-[contenthash:6].js",
			assetModuleFilename: "[contenthash][ext]"
		}
	},
	{
		...base,
		name: "b-source-map",
		context: path.resolve(__dirname, "b"),
		devtool: "source-map",
		output: {
			path: path.resolve(__dirname, "./dist/b-source-map"),
			filename: "[name].[contenthash]-[contenthash:6].js",
			assetModuleFilename: "[contenthash][ext]"
		}
	}
];
