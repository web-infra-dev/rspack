/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index"
	},
	target: "node",
	// output: {
	// 	filename: "[name].js"
	// },
	optimization: {
		splitChunks: {
			cacheGroups: {
				vendor: {
					chunks: "all",
					name: "vendor",
					test: "a.js"
				}
			}
		}
	}
};
