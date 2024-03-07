const rspack = require("@rspack/core");

/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	optimization: {
		minimize: true,
		splitChunks: false,
		chunkIds: "named"
	},
	plugins: [
		new rspack.SwcJsMinimizerRspackPlugin({
			extractComments: {
				filename: "[file].[base].COMMENTS.txt[query]"
			}
		})
	]
};
