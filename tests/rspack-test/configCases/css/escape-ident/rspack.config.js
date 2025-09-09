/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		css: true
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
				localIdentName: "[local]-[path]"
			}
		}
	}
};
