/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	cache: {
		type: "memory"
	},
	optimization: {
		sideEffects: false
	}
};
