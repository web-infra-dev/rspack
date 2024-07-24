/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: {
		colors: {
			yellow: "\u001b[33m",
			green: "\u001b[32m"
		}
	}
};
