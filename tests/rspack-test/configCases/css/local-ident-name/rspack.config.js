/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				localIdentName: "[local]-[hash:base64:6]-[hash]"
			}
		}
	},
	experiments: {
		css: true
	}
};
