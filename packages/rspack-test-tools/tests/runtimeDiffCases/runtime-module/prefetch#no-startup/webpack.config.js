/** @type {import("webpack").Configuration} */
module.exports = {
	target: "web",
	output: {
		chunkFilename: "[name].js",
		crossOriginLoading: "anonymous"
	}
};
