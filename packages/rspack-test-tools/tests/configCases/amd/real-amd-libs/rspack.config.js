/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		extensions: ["...", ".js"],
	},
	amd: { jQuery: true },
	externals: { jquery: 'global $' },
};
