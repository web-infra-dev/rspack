/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	},
	experiments: {
		css: true
	}
};
