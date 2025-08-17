/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	output: {
		library: {
			type: "module"
		}
	},
	experiments: {
		outputModule: true
	}
};
