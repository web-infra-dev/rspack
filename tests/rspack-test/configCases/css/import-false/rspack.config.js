/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	entry: {
		main: "./index.js"
	},
	module: {
		parser: {
			"css/auto": {
				resolveImport: false
			}
		},
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
