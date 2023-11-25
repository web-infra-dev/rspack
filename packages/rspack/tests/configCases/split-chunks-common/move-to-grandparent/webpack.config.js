/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index",
		misc: "./second"
	},
	output: {
		filename: "[name].js",
		chunkFilename: "async/[name].js" // avoid __webpack_require__.u fallback to default logic, it can run successfully
	},
	optimization: {
		splitChunks: {
			minSize: 0
		}
	}
};
