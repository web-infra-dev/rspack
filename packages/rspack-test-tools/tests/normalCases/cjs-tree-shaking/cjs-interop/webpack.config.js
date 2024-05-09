/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	optimization: {
		minimize: false
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: false
		}
	}
};
