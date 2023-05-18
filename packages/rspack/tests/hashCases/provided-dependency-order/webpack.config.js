const NodePolyfillPlugin = require("@rspack/plugin-node-polyfill");

/** @type {import("../../../").Configuration} */
module.exports = {
	mode: "development",
	devtool: false,
	output: {
		filename: "[name].[contenthash]-[contenthash:6].js"
	},
	optimization: {
		realContentHash: true
	},
	plugins: [new NodePolyfillPlugin()]
};
