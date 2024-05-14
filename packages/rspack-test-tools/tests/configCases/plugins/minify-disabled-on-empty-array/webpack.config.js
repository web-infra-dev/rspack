/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	},
	optimization: {
		minimize: true,
		minimizer: []
	}
};
