/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		removeAvailableModules: true,
		providedExports: true,
		usedExports: "global"
	},
};
