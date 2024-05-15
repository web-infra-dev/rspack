/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				localIdentName: "[path][name]-[local]",
			}
		}
	},
	mode: "development",
	experiments: {
		css: true
	}
};
