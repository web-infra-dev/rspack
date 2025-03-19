/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "named",
		splitChunks: {
			maxInitialRequests: 10,
			cacheGroups: {
				vendors: {
					chunks: "all",
					test: /node_modules/,
					minSize: 0,
					filename: "split-[name].js",

					// should override the splitChunks.maxInitialRequests
					maxInitialRequests: 1
				}
			}
		}
	}
};
