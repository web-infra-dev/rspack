/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			// hidePathInfo: false,
			minSize: 50,
			maxSize: 100
		}
	}
};
