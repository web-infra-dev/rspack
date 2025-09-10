/**
 * @type {import('@rspack/core').Configuration}
 */
module.exports = {
	entry: "./index.js",
	node: {
		__dirname: false,
		__filename: false
	},
	optimization: {
		sideEffects: false,
		concatenateModules: false,
		innerGraph: false
	}
};
