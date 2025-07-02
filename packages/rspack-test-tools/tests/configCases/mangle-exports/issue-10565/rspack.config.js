/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	optimization: {
		mangleExports: true,
		usedExports: true,
		providedExports: true,
		concatenateModules: false
	}
};
