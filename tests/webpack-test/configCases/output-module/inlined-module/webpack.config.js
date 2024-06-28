/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true
	},
	optimization: {
		concatenateModules: true
	},
	experiments: {
		outputModule: true
	},
	target: "es2020"
};
