/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./src/index.js"
	},
	optimization: {
		concatenateModules: true
	}
};
