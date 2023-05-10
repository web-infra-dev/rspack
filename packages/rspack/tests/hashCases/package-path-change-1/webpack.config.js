/** @type {import("../../../dist").Configuration} */
module.exports = {
	entry: "./index.js",
	context: __dirname,
	output: {
		filename: "[name].[contenthash].js"
	},
	optimization: {
		moduleIds: "named",
		minimize: false,
		runtimeChunk: {
			name: "runtime"
		}
	}
};
