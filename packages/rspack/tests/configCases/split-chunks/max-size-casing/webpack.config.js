/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	experiments: {
		newSplitChunks: true
	},
	optimization: {
		splitChunks: {
			hidePathInfo: false,
			minSize: 50,
			maxSize: 100
		}
	}
};
