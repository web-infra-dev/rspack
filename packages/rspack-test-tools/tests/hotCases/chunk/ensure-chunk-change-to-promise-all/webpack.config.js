/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	optimization: {
		splitChunks: {
			minSize: 0,
		}
	}
};
