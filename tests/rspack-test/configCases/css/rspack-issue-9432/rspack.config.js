/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	optimization: {
		concatenateModules: true
	},
	experiments: {
		css: true
	}
};
