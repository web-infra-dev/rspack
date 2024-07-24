/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	output: {
		chunkFilename: "[contenthash].js"
	}
};
