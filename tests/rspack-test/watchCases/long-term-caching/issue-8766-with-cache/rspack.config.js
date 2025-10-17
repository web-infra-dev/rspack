/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	cache: true,
	output: {
		chunkFilename: "[contenthash].js"
	}
};
