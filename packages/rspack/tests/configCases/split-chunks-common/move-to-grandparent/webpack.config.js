/** @type {import("../../../../").Configuration} */
module.exports = {
	entry: {
		main: "./index",
		misc: "./second"
	},
	output: {
		filename: "[name].js",
		chunkFilename: "async/[name].js" // avoid __webpack_require__.u fallback to default logic, it can run successfully
	},
	experiments: {
		newSplitChunks: true
	},
	optimization: {
		splitChunks: {
			minSize: 0
		}
	}
};
