/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	cache: true,
	output: {
		filename: "bundle.js?[contenthash]"
	}
};
