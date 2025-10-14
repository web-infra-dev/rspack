/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				lib1: {
					name: "lib1",
					test: /lib1/
				},
				lib2: {
					name: "lib2",
					test: /lib2/
				}
			}
		}
	}
};
