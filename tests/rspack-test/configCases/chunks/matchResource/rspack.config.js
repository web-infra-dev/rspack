/** @type {import('@rspack/core').Configuration} */
module.exports = (env, { testPath }) => ({
	output: {
		clean: true,
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				vendor: {
					chunks: "all",
					reuseExistingChunk: true,
					test: /[\\/]node_modules[\\/]/,
					minSize: 0,
					minChunks: 1
				}
			}
		}
	}
});
