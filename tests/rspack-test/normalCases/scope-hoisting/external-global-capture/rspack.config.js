/** @type {import("@rspack/core").Configuration} */
module.exports = {
	externals: {
		react: "var React"
	},
	optimization: {
		concatenateModules: true
	}
};
