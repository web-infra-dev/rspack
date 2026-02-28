/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		main: "./"
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				default: false,
				vendors: {
					chunks: "initial",
					filename: "[name].vendors.js",
					minSize: 1,
					maxInitialSize: 1,
					test: /[\\/]node_modules[\\/]/
				}
			}
		}
	},
	stats: {
		entrypoints: true,
		assets: false,
		chunks: true
	}
};
