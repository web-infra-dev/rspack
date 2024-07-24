
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./example.js",
	optimization: {
		usedExports: true,
		providedExports: true,
	},
	stats: {
		usedExports: true,
		providedExports: true,
	}
};

