/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	entry: {
		main: "./index.js"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		}
	},
	experiments: {
		css: true
	}
};
