/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	devtool: false,
	optimization: {
		concatenateModules: true,
		minimize: false
	}
};
