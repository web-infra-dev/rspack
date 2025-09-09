/** @type {import("webpack").Configuration} */
module.exports = {
	entry: {
		a: "./src/a/index.js",
		b: "./src/b/index.js",
		main: "./src/main/index.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: {
			name: "bundle"
		}
	},
	target: "node"
};
