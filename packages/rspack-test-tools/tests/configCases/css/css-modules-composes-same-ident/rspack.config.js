/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				localIdentName: "[path][name]-[local]"
			}
		}
	},
	experiments: {
		css: true,
	}
};
