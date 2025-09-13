/** @type {import("@rspack/core").Configuration} */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	optimization: {
		concatenateModules: false,
		minimize: false
	}
};
