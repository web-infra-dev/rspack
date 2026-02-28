/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		moduleIds: "named",
		usedExports: true,
		providedExports: true,
		concatenateModules: true
	}
};
